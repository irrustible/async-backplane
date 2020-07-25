#[cfg(feature = "smol")]
use smol::Task;
use async_channel::{self, Receiver, Sender};
use futures_lite::{Future, Stream, StreamExt};
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::{Crash, DeviceID, Disconnect, Error, LinkError};
use crate::linemap::LineMap;
use crate::plugboard::Plugboard;
use crate::utils::{biased_race, DontPanic};

/// A Device connects a Future to the backplane.
#[derive(Debug)]
pub struct Device {
    pub(crate) plugboard: Arc<Plugboard>,
    pub(crate) disconnects: Receiver<(DeviceID, Disconnect)>,
    monitors: LineMap,
    done: bool,
}

// The return type of `.monitoring()`
pub type Watching<T, C=Error> = Result<T, Result<(DeviceID, Disconnect), Crash<C>>>;

// The return type of `.part_manage()`
pub type PartManaging<T, C=Error> = Result<(Device, T), Crash<C>>;

// The return type of `.manage()`
pub type Managing<T, C=Error> = Result<T, Crash<C>>;

impl Device {

    /// Creates a new Device
    pub fn new() -> Self {
        let (send, disconnects) = async_channel::unbounded();
        let plugboard = Arc::new(Plugboard::new(send));
        Device {
            disconnects,
            plugboard,
            monitors: LineMap::new(),
            done: false,
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

    /// Notify our monitors that we were successful
    pub async fn completed(self) {
        self.disconnect(Disconnect::Complete).await;
    }

    /// Notify our monitors that we crashed
    pub async fn crashed(self) {
        self.disconnect(Disconnect::Crash).await;
    }

    /// Notify our monitors that we cascaded a crash
    pub async fn cascaded(self, did: DeviceID) {
        self.disconnect(Disconnect::Cascade(did)).await
    }

    /// Notify our monitors of our disconnect
    pub async fn disconnect(mut self, disconnect: Disconnect) {
        self.plugboard.broadcast(self.device_id(), disconnect).await;
        self.done = true;
    }

    /// Ask to be notified when the provided Line disconnects
    pub fn monitor(&self, line: Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            line.plugboard.attach(self.line(), LinkError::LinkDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Ask to not be notified when the provided Line disconnects
    pub fn demonitor(&self, line: &Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            line.plugboard.detach(self.device_id(), LinkError::LinkDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Notify the provided Line when we disconnect
    pub fn attach(&self, line: Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            self.plugboard.attach(line, LinkError::DeviceDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Undo attach
    pub fn detach(&self, did: DeviceID) -> Result<(), LinkError> {
        if self.device_id() != did {
            self.plugboard.detach(did, LinkError::DeviceDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Monitor + attach
    pub fn link(&self, line: Line) -> Result<(), LinkError> {
        self.monitor(line.clone())?;
        self.attach(line)?;
        Ok(())
    }

    /// Undo link
    pub fn unlink(&self, line: &Line) -> Result<(), LinkError> {
        self.detach(line.device_id())?;
        self.demonitor(line)?;
        Ok(())
    }

    /// Races the next disconnection from the Device and the provided
    /// future (which is wrapped to protect against crash)
    pub async fn watch<F, C>(
        &mut self,
        f: F
    ) -> Result<<F as Future>::Output, Result<(DeviceID, Disconnect), Crash<C>>>
    where F: Future + Unpin, C: 'static + Any + Send {
        let mut future = DontPanic::new(f);
        biased_race(
            async {
                let update = self.next().await.expect("The Device to still be usable.");
                Err(Ok(update))
            },
            async {
                match (&mut future).await {
                    Ok(val) => Ok(val),
                    Err(unwind) => Err(Err(Crash::Panic(unwind))),
                }
            }
        ).await
    }

    /// Runs an async closure while monitoring the self for crashes of
    /// any monitored Devices. If self (or a Device being monitored)
    /// crashes, announces that we have crashed to whoever is
    /// monitoring us. If it does not crash, returns the original
    /// Device for reuse along with the closure result.
    pub async fn part_manage<'a, F, T, C>(
        mut self, mut f: F
    ) -> Result<(Device, T), Crash<C>>
    where F: Future<Output=Result<T,C>> + Unpin, C: 'static + Send {
        loop {
            match self.watch(&mut f).await {
                Ok(Ok(val)) => { return Ok((self, val)); }
                Ok(Err(val)) => { return Err(Crash::Error(val)); }
                Err(Ok((did, disconnect))) => {
                    if disconnect.is_failure() {
                        self.cascaded(did).await;
                        return Err(Crash::Cascade(did, disconnect));
                    }
                }
                Err(Err(crash)) => {
                    self.disconnect(Disconnect::Crash).await;
                    return Err(crash);
                }
            }
        }
    }

    /// Like `part_manage()`, but in the case of successful
    /// completion, notifiers our monitors and consumes the `Device`.
    pub async fn manage<F, C, T>(self, f: F) -> Result<T, Crash<C>>
    where F: Future<Output=Result<T,C>> + Unpin, C: 'static + Send {
        match self.part_manage(f).await {
            Ok((device, val)) => {
                device.completed().await;
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }

    /// Like `manage()`, but in the case of a crash, reports it to the
    /// provided Sender instead of returning it.
    pub async fn fully_manage<F, C, T>(self, sender: Sender<(DeviceID, Crash<C>)>, f: F)
    where F: Future<Output=Result<T,C>> + Unpin, C: 'static + Send {
        let id = self.device_id();
        #[allow(unused_must_use)] // we don't check the Result
        if let Err(crash) = self.manage(f).await {
            sender.send((id, crash)).await; // not much to do if it fails
        }
    }

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

// #[cfg(feature = "smol")] // Send notifications in the background
// impl Drop for Device {
//     fn drop(&mut self) {
//         if !self.done {
//             let fut = self.plugboard.broadcast(self.device_id(), Disconnect::Crash);
//             Task::spawn(async move { fut.await; }).detach();
//         }
//     }
// }

// #[cfg(not(feature = "smol"))] // Block on notifications
impl Drop for Device {
    fn drop(&mut self) {
        use futures_lite::future::block_on;
        if !self.done {
            let fut = self.plugboard.broadcast(self.device_id(), Disconnect::Crash);
            block_on(fut);
        }
    }
}

impl Unpin for Device {}

impl Stream for Device {
    type Item = (DeviceID, Disconnect);
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        Receiver::poll_next(Pin::new(&mut Pin::into_inner(self).disconnects), ctx)
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

    /// Ask to be notified when the provided Line disconnects
    pub fn monitor(&self, line: Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            line.plugboard.attach(self.clone(), LinkError::LinkDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Ask to not be notified when the provided Line disconnects
    pub fn demonitor(&self, line: &Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            line.plugboard.detach(self.device_id(), LinkError::LinkDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Notify the provided Line when we disconnect
    pub fn attach(&self, line: Line) -> Result<(), LinkError> {
        if self.device_id() != line.device_id() {
            self.plugboard.attach(line, LinkError::DeviceDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Undo attach
    pub fn detach(&self, did: DeviceID) -> Result<(), LinkError> {
        if self.device_id() != did {
            self.plugboard.detach(did, LinkError::DeviceDown)
        } else {
            Err(LinkError::CantLinkSelf)
        }
    }

    /// Monitor + attach
    pub fn link(&self, line: Line) -> Result<(), LinkError> {
        self.monitor(line.clone())?;
        self.attach(line)?;
        Ok(())
    }

    /// Undo link
    pub fn unlink(&self, line: &Line) -> Result<(), LinkError> {
        self.detach(line.device_id())?;
        self.demonitor(line)?;
        Ok(())
    }
}

impl Eq for Line {}

impl Unpin for Line {}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        Arc::ptr_eq(&self.plugboard, &other.plugboard)
    }
}

