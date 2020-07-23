#[cfg(feature = "smol")]
use smol::Task;
use async_channel::{self, Receiver};
use futures_lite::{Future, Stream};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::{BulkSend, DeviceID, Disconnect, LinkError};
use crate::plugboard::Plugboard;

/// A Device is a computation's connection to the backplane
#[derive(Debug)]
pub struct Device {
    pub(crate) plugboard: Arc<Plugboard>,
    pub(crate) disconnects: Receiver<(DeviceID, Disconnect)>,
}

impl Device {

    /// Creates a new Device
    pub fn new() -> Self {
        let (send, disconnects) = async_channel::unbounded();
        let plugboard = Arc::new(Plugboard::new(send));
        Device { disconnects, plugboard }
    }

    // pub fn new_monitored(by: Line) -> Self {
    //     let (send, disconnects) = async_channel::unbounded();
    //     let plugboard = Arc::new(Plugboard::new(send));
    //     Device { disconnects, plugboard }
    // }

    /// Opens a line to the Device
    pub fn open_line(&self) -> Line {
        Line { plugboard: self.plugboard.clone() }
    }

    /// Notify our monitors that we were successful
    pub fn completed(self) -> BulkSend<(DeviceID, Disconnect)> {
        self.disconnect(Disconnect::Complete)
    }

    /// Notify our monitors that we crashed
    pub fn crashed(self) -> BulkSend<(DeviceID, Disconnect)> {
        self.disconnect(Disconnect::Crash)
    }

    /// Notify our monitors that we cascaded a crash
    pub fn cascaded(self, did: DeviceID) -> BulkSend<(DeviceID, Disconnect)> {
        self.disconnect(Disconnect::Cascade(did))
    }

    /// Notify our monitors of our disconnect
    pub fn disconnect(self, disconnect: Disconnect) -> BulkSend<(DeviceID, Disconnect)> {
        self.plugboard.broadcast(self.device_id(), disconnect)
    }

}

#[cfg(feature = "smol")]
impl Device {
    pub fn spawn<P, F>(&self, process: P) -> Line
    where P: FnOnce(Device) -> F,
          F: 'static + Future + Send
    {
        let device = Device::new();
        let line = device.open_line();
        let p = process(device);
        Task::spawn(async move { p.await; }).detach();
        line
    }
}

impl Unpin for Device {}

impl Device {
    /// Get the ID of the Device on the other end of the Line
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Ask to be notified when the provided Line disconnects
    pub fn monitor(&self, line: Line) -> Result<(), LinkError> {
        line.plugboard.attach(self.open_line(), LinkError::LinkDown)
    }

    /// Ask to not be notified when the provided Line disconnects
    pub fn demonitor(&self, line: &Line) -> Result<(), LinkError> {
        line.plugboard.detach(self.device_id(), LinkError::LinkDown)
    }

    /// Notify the provided Line when we disconnect
    pub fn attach(&self, line: Line) -> Result<(), LinkError> {
        self.plugboard.attach(line, LinkError::DeviceDown)
    }

    /// Undo attach
    pub fn detach(&self, did: DeviceID) -> Result<(), LinkError> {
        self.plugboard.detach(did, LinkError::DeviceDown)
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

impl Stream for Device {
    type Item = (DeviceID, Disconnect);
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        Receiver::poll_next(Pin::new(&mut Pin::into_inner(self).disconnects), ctx)
    }
}

/// A reference to a device that allows us to participate in monitoring
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
        line.plugboard.attach(self.clone(), LinkError::LinkDown)
    }

    /// Ask to not be notified when the provided Line disconnects
    pub fn demonitor(&self, line: &Line) -> Result<(), LinkError> {
        line.plugboard.detach(self.device_id(), LinkError::LinkDown)
    }

    /// Notify the provided Line when we disconnect
    pub fn attach(&self, line: Line) -> Result<(), LinkError> {
        self.plugboard.attach(line, LinkError::DeviceDown)
    }

    /// Undo attach
    pub fn detach(&self, did: DeviceID) -> Result<(), LinkError> {
        self.plugboard.detach(did, LinkError::DeviceDown)
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

