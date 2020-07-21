#[cfg(feature = "smol")]
use smol::Task;
use async_channel::{self, Receiver, Sender};
use futures_lite::{Future, Stream};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::{BulkSend, Crash, DeviceID, Disconnect, Line, LinkError, Managed, Monitoring, Pluggable};
use crate::plugboard::Plugboard;

/// A Device is a computation's connection to the backplane
pub struct Device {
    pub(crate) plugboard: Arc<Plugboard>,
    pub(crate) disconnects: Receiver<(DeviceID, Disconnect)>,
    pub(crate) crashes: Option<Sender<(DeviceID, Crash)>>,
}

impl Device {

    pub fn new(crashes: Sender<(DeviceID, Crash)>) -> Self {
        Device::new_with_crashes(Some(crashes))
    }

    pub fn new_with_crashes(crashes: Option<Sender<(DeviceID, Crash)>>) -> Self {
        let (send, disconnects) = async_channel::unbounded();
        let plugboard = Arc::new(Plugboard::new(send));
        Device { disconnects, plugboard, crashes }
    }

    // pub fn new_monitored(by: Line) -> Self {
    //     let (send, disconnects) = async_channel::unbounded();
    //     let plugboard = Arc::new(Plugboard::new(send));
    //     Device { disconnects, plugboard }
    // }

    pub fn open_line(&self) -> Line {
        Line { plugboard: self.plugboard.clone() }
    }

    pub fn disconnect(self, disconnect: Disconnect) -> BulkSend<(DeviceID, Disconnect)> {
        self.plugboard.broadcast(self.device_id(), disconnect)
    }

    pub fn monitoring<'a, F: Future>(&'a mut self, f: F) -> Monitoring<'a, F> {
        Monitoring::new(f, self)
    }

    pub fn managed<F: Future>(self, f: F) -> Managed<F> {
        Managed::new(f, self)
    }

}

#[cfg(feature = "smol")]
impl Device {
    pub fn spawn<P, F>(&self, process: P) -> Line
    where P: FnOnce(Device) -> F,
          F: 'static + Future + Send
    {
        let device = Device::new_with_crashes(self.crashes.clone());
        let line = device.open_line();
        let p = process(device);
        Task::spawn(async move { p.await; }).detach();
        line
    }

    pub fn spawn_blocking<P, F>(&self, process: P) -> Line
    where P: FnOnce(Device) -> F,
          F: 'static + Future + Send
    {
        let device = Device::new_with_crashes(self.crashes.clone());
        let line = device.open_line();
        let p = process(device);
        Task::blocking(async move { p.await; }).detach();
        line
    }

    pub fn spawn_local<P, F>(&self, process: P) -> Line
    where P: FnOnce(Device) -> F,
          F: 'static + Future
    {
        let device = Device::new_with_crashes(self.crashes.clone());
        let line = device.open_line();
        let p = process(device);
        Task::local(async move { p.await; }).detach();
        line
    }
}

impl Unpin for Device {}

impl Pluggable for Device {
    fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }
    fn monitor(&self, line: Line) -> Result<(), LinkError> {
        line.plugboard.attach(self.open_line(), LinkError::LinkDown)
    }
    fn demonitor(&self, line: &Line) -> Result<(), LinkError> {
        line.plugboard.detach(self.device_id(), LinkError::LinkDown)
    }
    fn attach(&self, line: Line) -> Result<(), LinkError> {
        self.plugboard.attach(line, LinkError::DeviceDown)
    }
    fn detach(&self, did: DeviceID) -> Result<(), LinkError> {
        self.plugboard.detach(did, LinkError::DeviceDown)
    }
    fn link(&self, line: Line) -> Result<(), LinkError> {
        self.monitor(line.clone())?;
        self.attach(line)?;
        Ok(())
    }
    fn unlink(&self, line: &Line) -> Result<(), LinkError> {
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
