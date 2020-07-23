#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]
use maybe_unwind::Unwind;
use std::any::Any;

mod plugboard;

mod device;
pub use device::{Device, Line};

pub mod utils;

pub mod panic;

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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
/// At least one end of the Link is down
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
pub enum Crash<C=FuckingAny> {

    /// If you installed the panic handler, this will be rich
    Panic(Unwind),
    /// Generically, something went wrong
    Fail(C),
    /// A device we depend upon disconnected
    Cascade(DeviceID, Disconnect),
}

impl<C> Crash<C> {
    /// is this an unwound panic?
    pub fn is_panic(&self) -> bool {
        if let Crash::Panic(_) = self {
            true
        } else {
            false
        }
    }

    /// is this the future returning Err?
    pub fn is_fail(&self) -> bool {
        if let Crash::Fail(_) = self {
            true
        } else {
            false
        }
    }

    /// is this crash in sympathy with another?
    pub fn is_cascade(&self) -> bool {
        if let Crash::Cascade(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Creates a disconnect representing this Crash.
    pub fn as_disconnect(&self) -> Disconnect {
        match self {
            Crash::Panic(_) => Disconnect::Crash,
            Crash::Fail(_) => Disconnect::Crash,
            Crash::Cascade(who, _) => Disconnect::Cascade(*who),
        }
    }
}

impl<C: 'static + Any + Send> Crash<C> {
    /// Boxes so you get no useful information whatsoever but the type
    /// is uniform. I HATE THIS.
    pub fn boxed(self) -> Crash<FuckingAny> {
        match self {
            Crash::Panic(unwind) => Crash::Panic(unwind),
            Crash::Fail(any) => Crash::Fail(Box::new(any)),
            Crash::Cascade(did, disco) => Crash::Cascade(did, disco),
        }
    }
}
