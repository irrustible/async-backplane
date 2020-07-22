#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]
use futures_lite::stream::StreamExt;
use maybe_unwind::{capture_panic_info, maybe_unwind, Unwind};
use std::any::Any;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};

mod sending;
pub use sending::BulkSend;

mod plugboard;

mod device;
pub use device::{Device, Line};

mod race;
use race::biased_race;

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

/// There is a problem with the link - at least one end of it is down.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LinkError {
    /// We can't because we are down
    DeviceDown,
    /// We can't because the other Device is down
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

/// Something we hope to replace very soon.
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
    pub fn as_disconnect(&self) -> Disconnect {
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

/// Wraps a Future such that it traps panics
pub struct DontPanic<F: Future> {
    fut: F,
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
        // pin_project!() cannot handle this scenario, it really has to be unsafe.
        let fut = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().fut) };
        match maybe_unwind(AssertUnwindSafe(|| fut.poll(ctx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Err(unwind) => Poll::Ready(Err(unwind))
        }
    }
}

/// Races the next disconnection from the Device and the provided
/// future (which is wrapped to protect against crash)
pub async fn monitoring<F: Future + Unpin, C: 'static + Any + Send>(
    device: &mut Device,
    f: F
) -> Result<<F as Future>::Output, Result<(DeviceID, Disconnect), Crash<C>>> {
    let mut future = DontPanic::new(f);
    biased_race(
        async {
            let update = device.next().await.unwrap();
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

/// Given a `Device` and an async closure, runs the async closure while
/// monitoring the `Device` for crashes of any monitored `Device`s.  If
/// the `Device` (or a `Device` being monitored) crashes, announces that
/// we have crashed to whoever is monitoring us. If it does not crash,
/// returns the original Device for reuse along with the closure result.
pub async fn part_manage<'a, F, T, C>(
    mut device: Device, mut f: F
) -> Result<(Device, T), Crash<C>>
where F: Future<Output=Result<T,C>> + Unpin,
      C: Any + 'static + Send
{
    loop {
        match monitoring(&mut device, &mut f).await {
            Ok(Ok(val)) => { return Ok((device, val)); }
            Ok(Err(val)) => { return Err(Crash::Fail(val)); }
            Err(Ok((did, disconnect))) => {
                if disconnect.crashed() {
                    device.cascaded(did).await;
                    return Err(Crash::Cascade(did, disconnect));
                }
            }
            Err(Err(crash)) => {
                device.disconnect(Disconnect::Crash).await;
                return Err(crash);
            }
        }
    }
}

/// Like `part_manage()`, but in the case of success, announces
/// success and consumes the `Device`.
pub async fn manage<'a, F, G, C, T>(device: Device, f: F)
                                    -> Result<T, Crash<C>>
where F: Future<Output=Result<T,C>> + Unpin,
      C: Any + 'static + Send
{
    match part_manage(device, f).await {
        Ok((device, val)) => {
            device.completed().await;
            Ok(val)
        }
        Err(e) => Err(e),
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
    
