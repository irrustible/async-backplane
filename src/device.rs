#[cfg(feature = "smol")]
use smol::Task;
use concurrent_queue::PopError;
use futures_lite::{Future, Stream, StreamExt};
use std::any::Any;
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::*;
use crate::linemap::LineMap;
use crate::plugboard::Plugboard;
use crate::utils::{biased_race, DontPanic};
use std::fmt::Debug;

/// A Device connects a Future to the backplane.
#[derive(Debug)]
pub struct Device {
    plugboard: Arc<Plugboard>,
    // This is here so we don't have to mark everything
    // mut. Accordingly, we also can't let the user have direct
    // access, in case they e.g. hold it across an await boundary.
    inner: RefCell<Inner>,
}

#[derive(Debug)]
struct Inner {
    out: LineMap,
    done: bool,
}

impl Inner {
    // Actually send all the messages.
    fn send(&mut self, message: Message) {
        let mut last: Option<Message> = None; // avoid copying
        for (_, maybe) in self.out.drain() {
            if let Some(line) = maybe {
                let m = last.take().unwrap_or_else(|| message.clone());
                if let Err(e) = line.send(m) { last = Some(e); }
            }
        }
    }
}

/// A result from `watch()`.
#[derive(Debug)]
pub enum Watched<T: Debug> {
    /// The provided Future completed.
    Completed(T),
    /// A message was received.
    Messaged(Message),
}

use Watched::{Completed, Messaged};

impl<T: Debug> Watched<T> {

    /// True if the future completed.
    pub fn is_completed(&self) -> bool {
        if let Messaged(_) = self { true } else { false }
    }

    /// True if we received a message.
    pub fn is_messaged(&self) -> bool {
        if let Messaged(_) = self { true } else { false }
    }

    /// Take the completed result or panic.
    pub fn unwrap_completed(self) -> T {
        if let Completed(c) = self { c }
        else { panic!("Watched is not Completed"); }
    }

    /// Take the received message or panic.
    pub fn unwrap_messaged(self) -> Message {
        if let Messaged(m) = self { m }
        else { panic!("Watched is not Messaged"); }
    }
}

impl<T: Debug + PartialEq> PartialEq for Watched<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Completed(l), Completed(r)) => *l == *r,
            (Messaged(l), Messaged(r)) => *l == *r,
            _ => false,
        }
    }
}

impl<T: Debug + Eq> Eq for Watched<T> {}

impl Device {

    /// Creates a new Device.
    pub fn new() -> Self {
        Device {
            plugboard: Arc::new(Plugboard::new()),
            inner: RefCell::new(Inner { out: LineMap::new(), done: false }),
        }
    }

    /// Get the ID of this Device.
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Opens a line to the Device.
    pub fn line(&self) -> Line {
        Line { plugboard: self.plugboard.clone() }
    }

    /// Notify our peers we're disconnecting.
    pub fn disconnect(self, fault: Option<Fault>) {
        self.do_disconnect(fault);
    }

    fn do_disconnect(&self, fault: Option<Fault>) {
        self.plugboard.close(); // no more requests
        let mut inner = self.inner.borrow_mut();
        while let Ok(op) = self.plugboard.line_ops.pop() { inner.out.apply(op); } // sync
        inner.send(Disconnected(Report::new(self.device_id(), fault)));
    }

    /// Link with another Device with the provided LinkMode. LinkModes
    /// are additive, so you can 'upgrade' a link this way.
    ///
    /// This method is intended for static-style linking, where the
    /// topology is not expected to change. You should not link to a
    /// Device this way after linking to it through a Line.
    pub fn link(&self, other: &Device, mode: LinkMode) {
        if self.device_id() != other.device_id() {
             if mode.monitor() {
                 other.inner.borrow_mut().out
                     .attach(Line { plugboard: self.plugboard.clone() });
             }
             if mode.notify() {
                 self.inner.borrow_mut().out
                     .attach(Line { plugboard: other.plugboard.clone() });
             }
        } else {
            panic!("Do not link to yourself!");
        }
    }

    /// Unlink from another Device with the provided LinkMode. LinkModes
    /// are subtractive, so you can 'downgrade' a link this way.
    ///
    /// This method is intended for static-style linking, where the
    /// topology is not expected to change. You should not link to a
    /// Device this way after linking to it through a Line.
    pub fn unlink(&self, other: &Device, mode: LinkMode) {
        if self.device_id() != other.device_id() {
            if mode.monitor() {
                other.inner.borrow_mut().out.detach(self.device_id());
            }
            if mode.notify() {
                self.inner.borrow_mut().out.detach(other.device_id());
            }
        } else {
            panic!("Do not link to yourself!");
        }
    }
   
    /// Link with a line. This is safer than linking directly to a
    /// Device, but a little slower.
    pub fn link_line(&self, other: Line, mode: LinkMode) -> Result<(), LinkError>{
        if self.device_id() != other.device_id() {
            if mode.monitor() {
                other.plugboard.plug(self.line(), LinkError::LinkDown)?;
            }
            if mode.notify() {
                self.inner.borrow_mut().out.attach(other);
            }
            Ok(())
        } else {
            panic!("Do not link to yourself!");
        }
    }

    /// Unlink with a line. This is safer than linking directly to a
    /// Device, but a little slower.
    pub fn unlink_line(&self, other: &Line, mode: LinkMode) {
        if self.device_id() != other.device_id() {
            #[allow(unused_must_use)]
            if mode.monitor() {
                other.plugboard.unplug(self.device_id(), LinkError::LinkDown);
            }
            if mode.notify() {
                self.inner.borrow_mut().out.detach(other.device_id());
            }
        } else {
            panic!("Do not link to yourself!");
        }
    }

    /// Returns the first of (with a bias towards the former):
    /// * The next message to be received.
    /// * The result of the completed future.
    /// * The crash of the Device.
    pub async fn watch<F, C>(&mut self, f: F)
                             -> Result<Watched<<F as Future>::Output>, Crash<C>>
    where F: Future + Unpin,
          F::Output: Debug,
          C: 'static + Any + Debug + Send {
        let mut future = DontPanic::new(f);
        biased_race(
            async {
                let message = self.next().await.expect("The Device to still be usable.");
                Ok(Messaged(message))
            },
            async {
                match (&mut future).await {
                    Ok(val) => Ok(Completed(val)),
                    Err(unwind) => Err(Crash::Panic(unwind)),
                }
            }
        ).await
    }

    /// Runs an async closure while monitoring for messages. Messages
    /// are handled as follows:
    ///
    /// * Disconnects without fault are ignored.
    /// * Disconnects with fault cause the Device to fault.
    /// * Requests to disconnect cause the Device to crash but
    /// announce a successful completion.
    ///
    /// If the provided closure returns successfully, the result is
    /// returned along with the Device for re-use. Monitors will *not*
    /// be notified.
    ///
    /// If the Device faults, either because the provided closure
    /// returned an Err variant or because a fault was propagated,
    /// announces our fault to our monitors.
    pub async fn part_manage<'a, F, T, C>(mut self, mut f: F)
                                          -> Result<(Device, T), Crash<C>>
    where F: Future<Output = Result<T, C>> + Unpin,
          C: 'static + Debug + Send,
          T: Debug {
        loop {
            match self.watch(&mut f).await {
                Ok(Completed(Ok(val))) => { return Ok((self, val)); }
                Ok(Completed(Err(val))) => {
                    self.disconnect(Some(Fault::Error));
                    return Err(Crash::Error(val));
                }
                Ok(Messaged(Disconnected(disco))) => {
                    if let Some(fault) = disco.result {
                        self.disconnect(Some(Fault::Cascade(disco.device_id)));
                        return Err(Crash::Cascade(Report::new(disco.device_id, fault)));
                    } else {
                        #[allow(unused_must_use)]
                        if !self.inner.borrow_mut().out.detach(self.device_id()) {
                            self.plugboard.unplug(self.device_id(), LinkError::LinkDown);
                        }
                        continue;
                    }
                }
                Ok(Messaged(Shutdown(id))) => {
                    self.disconnect(None);
                    return Err(Crash::PowerOff(id));
                }
                Err(crash) => {
                    self.disconnect(Some(Fault::Error));
                    return Err(crash);
                }
            }
        }
    }

    /// Like `part_manage()`, but in the case of successful completion
    /// of the provided future, notifies our monitors and consumes self
    pub async fn manage<F, C, T>(self, f: F) -> Result<T, Crash<C>>
    where F: Future<Output=Result<T,C>> + Unpin,
          C: 'static + Debug + Send,
          T: Debug {
        match self.part_manage(f).await {
            Ok((device, val)) => {
                device.disconnect(None);
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }

}

#[cfg(feature = "smol")]
impl Device {
    /// Spawns a computation with the Device on the global executor.
    ///
    /// Note: Requires the 'smol' feature (default enabled).
    pub fn spawn<P, F>(self, process: P) -> Line
    where P: FnOnce(Device) -> F,
          F: 'static + Future + Send
    {
        let line = self.line();
        let p = process(self);
        Task::spawn(async move { p.await; }).detach();
        line
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        if !inner.done {
            self.plugboard.close(); // no more requests
            while let Ok(op) = self.plugboard.line_ops.pop() { inner.out.apply(op); } // sync
            inner.send(Disconnected(Report::new(self.device_id(), Some(Fault::Drop))));
        }
    }
 }

impl Unpin for Device {}

impl Stream for Device {
    type Item = Message;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut inner = this.inner.borrow_mut();
        if !inner.done {
            match this.plugboard.messages.try_pop() {
                Ok(val) => Poll::Ready(Some(val)),
                Err(PopError::Empty) => {
                    this.plugboard.messages.register(ctx.waker());
                    // Make sure we don't lose out in a race
                    match this.plugboard.messages.try_pop() {
                        Ok(val) => Poll::Ready(Some(val)), // Sorry for leaving a waker
                        Err(PopError::Empty) => Poll::Pending,
                        Err(PopError::Closed) => {
                            inner.done = true;
                            Poll::Ready(None)
                        }
                    }
                }
                Err(PopError::Closed) => {
                    inner.done = true;
                    Poll::Ready(None)
                }
            }
        } else {
            Poll::Ready(None)
        }
    }
}

/// A reference to a `Device` that allows us to link with it.
#[derive(Clone, Debug)]
pub struct Line {
    pub(crate) plugboard: Arc<Plugboard>,
}

impl Line {
    /// Get the ID of the Device this line is connected to.
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Send a message to the Device.
    pub fn send(self, message: Message) -> Result<(), Message> {
        self.plugboard.send(message)
    }

    /// Links with another Line.
    pub fn link_line(&self, other: Line, mode: LinkMode) -> Result<(), LinkError>{
        if self.device_id() != other.device_id() {
            if mode.monitor() {
                other.plugboard.plug(self.clone(), LinkError::LinkDown)?;
            }
            if mode.notify() {
                self.plugboard.plug(other, LinkError::DeviceDown)?;
            }
            Ok(())
        } else {
            panic!("Do not link to yourself.");
        }
    }

    /// Links with another Line.
    pub fn unlink_line(&self, other: &Line, mode: LinkMode) {
        if self.device_id() != other.device_id() {
            #[allow(unused_must_use)]
            if mode.monitor() {
                other.plugboard.unplug(self.device_id(), LinkError::LinkDown);
            }
            #[allow(unused_must_use)]
            if mode.notify() {
                self.plugboard.unplug(other.device_id(), LinkError::DeviceDown);
            }
        }
    }
}

impl Eq for Line {}

impl Unpin for Line {}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        Arc::ptr_eq(&self.plugboard, &other.plugboard)
    }
}
