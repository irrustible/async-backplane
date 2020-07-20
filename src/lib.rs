use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;
use async_channel::Receiver;
use futures_lite::stream::Stream;
use maybe_unwind::{capture_panic_info, maybe_unwind, Unwind};
use std::panic;
use pin_project_lite::pin_project;

mod plugboard;
use plugboard::Plugboard;

/// A locally unique identifier for a Device
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeviceID {
    pub(crate) inner: usize,
}

impl DeviceID {
    pub(crate) fn new(inner: usize) -> DeviceID {
        DeviceID { inner }
    }
}

pub trait Pluggable {
    fn device_id(&self) -> DeviceID;
    /// Ask to be notified when the provided Line disconnects
    fn monitor(&self, line: Line) -> Result<(), LinkError>;
    /// Ask to not be notified when the provided Line disconnects
    fn demonitor(&self, line: &Line) -> Result<(), LinkError>;
    /// Notify the provided Line when we disconnect
    fn attach(&self, line: Line) -> Result<(), LinkError>;
    /// Undo attach
    fn detach(&self, device_id: DeviceID) -> Result<(), LinkError>;
    /// Monitor + attach
    fn link(&self, line: Line) -> Result<(), LinkError>;
    /// Undo link
    fn unlink(&self, line: &Line) -> Result<(), LinkError>;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LinkError {
    DeviceDown,
    LinkDown,
}

/// The device has dropped off the bus
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Disconnect {
    Complete,
    Crash,
    /// A device we depended on errored
    Cascade(DeviceID),
}

impl Disconnect {
    fn completed(&self) -> bool { *self == Disconnect::Complete }
    fn crashed(&self) -> bool { !self.completed() }
}

/// Something went wrong with a Device
#[derive(Debug)]
pub enum Crash<C=()> {
    Panic(Unwind),
    Fail(C),
    Cascade(DeviceID, Disconnect),
}

impl<C> Crash<C> {
    pub fn try_convert<D>(self) -> Result<Crash<D>, Crash<C>> {
        match self {
            Crash::Panic(unwind) => Ok(Crash::Panic(unwind)),
            Crash::Cascade(did, disco) => Ok(Crash::Cascade(did, disco)),
            _ => Err(self),
        }
    }
}

/// A Device is a computation's connection to the backplane
pub struct Device {
    disconnects: Receiver<(DeviceID, Disconnect)>,
    plugboard: Arc<Plugboard>,
}

impl Device {
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

    pub fn open_line(&self) -> Line {
        Line { plugboard: self.plugboard.clone() }
    }

    pub fn disconnect(self, disconnect: Disconnect) {
        self.plugboard.broadcast(self.device_id(), disconnect);
    }
}

// #[cfg(feature = "smol")]
// impl Device {
//     pub fn spawn<E>(fut: F) -> Line
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let device = Device::new(fut);
//         let line = device.as_line();
//         Task::spawn(DeviceTask { device }).detach();

//         line
//     }

            //     pub fn spawn_blocking<E>(fut: F) -> Line
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let device = Device::new(fut);
//         let line = device.as_line();
//         Task::blocking(DeviceTask { device }).detach();

//         line
//     }

//     pub fn spawn_local<E>(fut: F) -> Line
//     where
//         F: Future<Output = Result<(), E>> + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let device = Device::new(fut);
//         let line = device.as_line();
//         Task::local(DeviceTask { device }).detach();

//         line
//     }
// }

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

#[derive(Clone)]
pub struct Line {
    pub(crate) plugboard: Arc<Plugboard>,
}

impl Eq for Line {}

impl Unpin for Line {}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        Arc::ptr_eq(&self.plugboard, &other.plugboard)
    }
}

impl Pluggable for Line {
    fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }
    fn monitor(&self, line: Line) -> Result<(), LinkError> {
        line.plugboard.attach(self.clone(), LinkError::LinkDown)
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

/// Wraps a Future such that it catches panics when being polled.
pub struct DontPanic<F: Future> {
    fut: F,
}

impl<F, T> Future for DontPanic<F>
where F: Future<Output=T> {
    type Output = Result<T, Unwind>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        // pin_project!() cannot handle this scenario, it really has to be unsafe.
        let fut = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().fut) };
        match maybe_unwind(AssertUnwindSafe(|| fut.poll(ctx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Err(unwind) => Poll::Ready(Err(unwind))
        }
    }
}

pin_project! {
    /// Wraps a Future such that it will return `Err(Crash<C>)` if it
    /// crashes or one of the Futures it is monitoring crashes.
    pub struct Monitoring<'a, F: Future> {
        #[pin]
        fut: DontPanic<F>,
        device: Option<&'a mut Device>
    }
}

impl<'a, F, C, T> Future for Monitoring<'a, F>
where F: Future<Output=Result<T, C>> {
    type Output = Result<T, Crash<C>>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(ref mut device) = &mut this.device {
            loop {
                match Device::poll_next(Pin::new(device), ctx) {
                    Poll::Ready(Some((id, disconnect))) => {
                        if disconnect.crashed() {
                            return Poll::Ready(Err(Crash::Cascade(id, disconnect)));
                        }
                    }
                    Poll::Pending => {
                        return match DontPanic::poll(this.fut, ctx) {

                            Poll::Pending => Poll::Pending,

                            Poll::Ready(Ok(Ok(val))) => Poll::Ready(Ok(val)),

                            Poll::Ready(Ok(Err(val))) =>
                                Poll::Ready(Err(Crash::Fail(val))),

                            Poll::Ready(Err(unwind)) =>
                                Poll::Ready(Err(Crash::Panic(unwind))),

                        }
                    }
                    Poll::Ready(None) => unreachable!(),
                }
            }
        } else {
            Poll::Pending // We have already completed
        }
    }
}

// pub struct Monitored<F: Future> {
//     fut: DontPanic<F>,
//     device: Option<Device>,
// }

// impl<F, C, T> Future for Monitored<'a, F>
// where F: Future<Output=Result<T, C>> {
//     type Output = Result<T, Crash<C>>;
//     fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
//         let mut this = self.project();
//         if let Some(ref mut device) = &mut this.device {
//             loop {
//                 match Device::poll_next(Pin::new(device), ctx) {
//                     Poll::Ready(Some((id, disconnect))) => {
//                         if disconnect.crashed() {
//                             return Poll::Ready(Err(Crash::Cascade(id, disconnect)));
//                         }
//                     }
//                     Poll::Pending => {
//                         return match DontPanic::poll(this.fut, ctx) {

//                             Poll::Pending => Poll::Pending,

//                             Poll::Ready(Ok(Ok(val))) => Poll::Ready(Ok(val)),

//                             Poll::Ready(Ok(Err(val))) =>
//                                 Poll::Ready(Err(Crash::Fail(val))),

//                             Poll::Ready(Err(unwind)) =>
//                                 Poll::Ready(Err(Crash::Panic(unwind))),

//                         }
//                     }
//                     Poll::Ready(None) => unreachable!(),
//                 }
//             }
//         } else {
//             Poll::Pending // We have already completed
//         }
//     }
// }

// pub trait Reporter {
//     fn report(&mut self, device: &mut Device, &mut Crash,
// }



// pub struct Crashy<F: Future> {
//     fut: F,
//     device: Option<Device>,
// }

// impl<F, C, T> Future for Supervised<F>
// where F: Future<Output=Result<T, C>> {
//     type Output = Result<T, Crash<C>>;
//     fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
//         let this = unsafe { self.get_unchecked_mut() };
//         if let Some(ref mut device) = &mut this.device {
//             loop {
//                 match Device::poll_next(Pin::new(device), ctx) {
//                     Poll::Ready(Some((id, disconnect))) => {
//                         match disconnect {
//                             Disconnect::Crash => {
//                                 let disco = Disconnect::Cascade(device.device_id());
//                                 device.plugboard.broadcast(device.device_id(), disco);
//                                 return Poll::Ready(Err(Crash::Cascade(id)));
//                             }
//                             Disconnect::Cascade(from) => {
//                                 device.plugboard.broadcast(device.device_id(), disconnect);
//                                 return Poll::Ready(Err(Crash::Cascade(from)));
//                             }
//                             _ => () // spin
//                         }
//                     }
//                     Poll::Pending => {
//                         let fut = unsafe { Pin::new_unchecked(&mut this.fut) };
//                         return match maybe_unwind(AssertUnwindSafe(|| fut.poll(ctx))) {
//                             Ok(Poll::Pending) => Poll::Pending,
//                             Ok(Poll::Ready(Ok(val))) => {
//                                 device.plugboard.broadcast(device.device_id(), Disconnect::Complete);
//                                 return Poll::Ready(Ok(val))
//                             }
//                             Ok(Poll::Ready(Err(val))) => {
//                                 device.plugboard.broadcast(device.device_id(), Disconnect::Crash);
//                                 return Poll::Ready(Err(Crash::Fail(val.into())))
//                             }
//                             Err(unwind) => {
//                                 device.plugboard.broadcast(device.device_id(), Disconnect::Crash);
//                                 return Poll::Ready(Err(Crash::Panic(unwind)));
//                             }
//                         }
//                     }
//                     Poll::Ready(None) => { return Poll::Pending; } // shouldn't happen
//                 }
//             }
//         }
//         Poll::Pending
//     }
// }

/// Sets the thread local panic handler to record the unwind information
pub fn replace_panic_hook() {
    panic::set_hook(Box::new(|info| { capture_panic_info(info); }));
}

/// Sets the thread local panic handler to record the unwind information
/// and then execute whichever other hook was already in place
pub fn chain_panic_hook() {
    let old = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        capture_panic_info(info);
        old(info);
    }));
}
