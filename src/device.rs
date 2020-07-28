#[cfg(feature = "smol")]
use smol::Task;
use concurrent_queue::PopError;
use futures_lite::{Future, Stream, StreamExt};
use std::any::Any;
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::{Crash, DeviceID, Disconnect, Fault, Error, LinkError, LinkMode, Or, Report};
use crate::linemap::LineMap;
use crate::plugboard::Plugboard;
use crate::utils::{biased_race, DontPanic};

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
    done: bool
}

impl Inner {
    // Actually send all the messages
    fn report(&mut self, report: Report<Option<Fault>>) {
        let mut last: Option<Report<Option<Fault>>> = None; // avoid copying
        for (_, maybe) in self.out.drain() {
            if let Some(line) = maybe {
                let r = last.take().unwrap_or_else(|| report.clone());
                if let Err(e) = line.report(r) { last = Some(e); }
            }
        }
    }
}

// The return type of `.watch()`
pub type Watch<T, C=Error> = Result<Or<T, Disconnect>, Crash<C>>;

// The return type of `.part_manage()`
pub type PartManage<T, C=Error> = Result<(Device, T), Crash<C>>;

// The return type of `.manage()`
pub type Manage<T, C=Error> = Result<T, Crash<C>>;

impl Device {

    /// Creates a new Device
    pub fn new() -> Self {
        Device {
            plugboard: Arc::new(Plugboard::new()),
            inner: RefCell::new(Inner { out: LineMap::new(), done: false }),
        }
    }

    /// Get the ID of the Device on the other end of the Line
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Opens a line to the Device
    pub fn line(&self) -> Line {
        Line { plugboard: self.plugboard.clone() }
    }

    /// Notify our peers we're disconnecting
    pub fn disconnect(self, fault: Option<Fault>) {
        self.do_disconnect(fault);
    }

    fn do_disconnect(&self, fault: Option<Fault>) {
        self.plugboard.close(); // no more requests
        let mut inner = self.inner.borrow_mut();
        while let Ok(op) = self.plugboard.line_ops.pop() { inner.out.apply(op); } // sync
        inner.report(Report::new(self.device_id(), fault));
    }

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

    #[allow(unused_must_use)]
    pub fn unlink_line(&self, other: &Line, mode: LinkMode) {
        if self.device_id() != other.device_id() {
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

    /// Races the next disconnection from the Device and the provided
    /// future (which is wrapped to protect against crash)
    pub async fn watch<F, C>(&mut self, f: F) -> Watch<<F as Future>::Output, C>
    where F: Future + Unpin, C: 'static + Any + Send {
        let mut future = DontPanic::new(f);
        biased_race(
            async {
                let update = self.next().await.expect("The Device to still be usable.");
                Ok(Or::Right(update))
            },
            async {
                match (&mut future).await {
                    Ok(val) => Ok(Or::Left(val)),
                    Err(unwind) => Err(Crash::Panic(unwind)),
                }
            }
        ).await
    }

    /// Runs an async closure while monitoring the self for crashes of
    /// any monitored Devices. If self (or a Device being monitored)
    /// crashes, announces that we have crashed to whoever is
    /// monitoring us. If it does not crash, returns the original
    /// Device for reuse along with the closure result.
    pub async fn part_manage<'a, F, T, C>(mut self, mut f: F) -> PartManage<T, C>
    where F: Future<Output = Result<T, C>> + Unpin, C: 'static + Send {
        loop {
            match self.watch(&mut f).await {
                Ok(Or::Left(Ok(val))) => {
                    #[allow(unused_must_use)]
                    if !self.inner.borrow_mut().out.detach(self.device_id()) {
                        self.plugboard.unplug(self.device_id(), LinkError::LinkDown);
                    }
                    return Ok((self, val));
                }
                Ok(Or::Right(disco)) => {
                    if let Some(fault) = disco.result {
                        self.disconnect(Some(Fault::Cascade(disco.device_id)));
                        return Err(Crash::Cascade(Report::new(disco.device_id, fault)));
                    }
                    continue;
                }
                Ok(Or::Left(Err(val))) => {
                    self.disconnect(Some(Fault::Error));
                    return Err(Crash::Error(val));
                }
                Err(crash) => {
                    self.disconnect(Some(Fault::Error));
                    return Err(crash);
                }
            }
        }
    }

    /// Like `part_manage()`, but in the case of successful
    /// completion, notifies our monitors and consumes self
    pub async fn manage<F, C, T>(self, f: F) -> Result<T, Crash<C>>
    where F: Future<Output=Result<T,C>> + Unpin, C: 'static + Send {
        match self.part_manage(f).await {
            Ok((device, val)) => {
                device.disconnect(None);
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }

    // /// Like `manage()`, but in the case of a crash, reports it to the
    // /// provided Sender instead of returning it.
    // pub async fn fully_manage<F, C, T>(self, sender: Sender<Report<Crash<C>>>, f: F)
    // where F: Future<Output=Result<T,C>> + Unpin, C: 'static + Send {
    //     let id = self.device_id();
    //     #[allow(unused_must_use)] // we don't check the Result
    //     if let Err(crash) = self.manage(f).await {
    //         sender.send(Report::new(id, crash));
    //     }
    // }

}

#[cfg(feature = "smol")]
impl Device {
    /// Spawns a computation with the Device on the global executor.
    ///
    /// Note: Requires the 'smol' feature (default enabled)
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
            inner.report(Report::new(self.device_id(), Some(Fault::Drop)));
        }
    }
 }

impl Unpin for Device {}

impl Stream for Device {
    type Item = Disconnect;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut inner = this.inner.borrow_mut();
        if !inner.done {
            match this.plugboard.disconnects.try_pop() {
                Ok(val) => Poll::Ready(Some(val)),
                Err(PopError::Empty) => {
                    this.plugboard.disconnects.register(ctx.waker());
                    // Make sure we don't lose out in a race
                    match this.plugboard.disconnects.try_pop() {
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

/// A reference to a device that allows us to monitor it, be monitored
/// by it or link with it (both monitor and be monitored).
#[derive(Clone, Debug)]
pub struct Line {
    pub(crate) plugboard: Arc<Plugboard>,
}

impl Line {
    /// Get the ID of the Device on the other end of the Line
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Report disconnection
    pub fn report(self, disconnect: Disconnect) -> Result<(), Disconnect> {
        self.plugboard.notify(disconnect)
    }

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
            Err(LinkError::CantLinkSelf)
        }
    }

    #[allow(unused_must_use)]
    pub fn unlink_line(&self, other: &Line, mode: LinkMode) {
        if self.device_id() != other.device_id() {
            if mode.monitor() {
                other.plugboard.unplug(self.device_id(), LinkError::LinkDown);
            }
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
