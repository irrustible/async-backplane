#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]
use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;
use maybe_unwind::{capture_panic_info, maybe_unwind, Unwind};
use std::panic;
use pin_project_lite::pin_project;

mod sending;
pub use sending::BulkSend;

mod plugboard;
use plugboard::Plugboard;

mod device;
pub use device::Device;

mod monitoring;
pub use monitoring::Monitoring;

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
    /// Success!
    Complete,
    /// The device errored
    Crash,
    /// A device we depended on errored
    Cascade(DeviceID),
}

impl Disconnect {
    /// Whether the disconnect was a successful completion
    fn completed(&self) -> bool { *self == Disconnect::Complete }
    /// Whether the disconnect was a crash (or a cascade, which counts)
    fn crashed(&self) -> bool { !self.completed() }
}

pub type FuckingAny = Box<dyn Any + 'static + Send>;

/// Something went wrong with a Device
#[derive(Debug)]
pub enum Crash<C=FuckingAny>
where C: 'static + Send {
    /// If you installed the panic handler, this will be rich
    Panic(Unwind),
    /// Generically, something went wrong
    Fail(C),
    /// A device we depend upon disconnected
    Cascade(DeviceID, Disconnect),
}

impl<C: Any + 'static + Send> Crash<C> {
    fn as_disconnect(&self) -> Disconnect {
        match self {
            Crash::Panic(_) => Disconnect::Crash,
            Crash::Fail(_) => Disconnect::Crash,
            Crash::Cascade(who, _) => Disconnect::Cascade(*who),
        }
    }
}

impl<C: 'static + Any + Send> Crash<C> {
    pub fn boxed(self) -> Crash<FuckingAny> {
        match self {
            Crash::Panic(unwind) => Crash::Panic(unwind),
            Crash::Fail(any) => Crash::Fail(Box::new(any)),
            Crash::Cascade(did, disco) => Crash::Cascade(did, disco),
        }
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

pin_project! {
    /// Wraps a Future such that it traps panics
    pub struct DontPanic<F: Future> {
        #[pin]
        fut: F,
    }
}

impl<F: Future> DontPanic<F> {
    fn new(fut: F) -> Self {
        DontPanic { fut }
    }
}

impl<F, T> Future for DontPanic<F>
where F: Future<Output=T> {
    type Output = Result<T, Unwind>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        match maybe_unwind(AssertUnwindSafe(|| this.fut.poll(ctx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Err(unwind) => Poll::Ready(Err(unwind))
        }
    }
}

/// Runs a provided async closure as Monitoring, but relays disconnects to it
pub async fn managed<'a, F, G, C, T>(mut device: Device, f: F)
                                     -> Result<T, (DeviceID, Crash<C>)>
where F: FnOnce() -> G,
      G: Future<Output=Result<T,C>>,
      C: Any + 'static + Send
{
    let did = device.device_id();
    match device.monitoring(f()).await {
        Ok(val) => {
            device.plugboard.broadcast(did, Disconnect::Complete).await;
            Ok(val)
        }
        Err(crash) =>  {
            device.plugboard.broadcast(did, crash.as_disconnect()).await;
            Err((did, crash))
        }
    }
}

// pub trait Reporter {
//     fn report(&mut self, device: &mut Device, &mut Crash)
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
