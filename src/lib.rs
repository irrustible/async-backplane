#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

pub mod utils;
pub mod panic;

mod plugboard;
mod device;

pub use anyhow::{anyhow as error, bail as crash, ensure, Error};
pub use device::{Device, Line};

use maybe_unwind::Unwind;
use std::fmt::Display;

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

    /// Whether the disconnect was a successful completion.
    pub fn is_complete(&self) -> bool { *self == Disconnect::Complete }

    /// Whether the disconnect was a crash.
    pub fn is_crash(&self) -> bool { *self == Disconnect::Crash }

    /// Whether the disconnect was a cascade.
    pub fn is_cascade(&self) -> bool {
        if let Disconnect::Cascade(_) = self {
            true
        } else {
            false
        }
    }
    
    /// Whether the disconnect was a crash or cascade
    pub fn is_failure(&self) -> bool { !self.is_complete() }
}


/// Something went wrong with a Device
#[derive(Debug)]
pub enum Crash<C=Error> {
    /// The Device panicked.
    Panic(Unwind),
    /// The Device returned an Err
    Error(C),
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
    pub fn is_error(&self) -> bool {
        if let Crash::Error(_) = self {
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
            Crash::Error(_) => Disconnect::Crash,
            Crash::Cascade(who, _) => Disconnect::Cascade(*who),
        }
    }

}

impl Crash {
    /// If we are an Error, add additional context. Otherwise, do nothing.
    pub fn with_context<Ctx>(self, context: Ctx) -> Self
    where Ctx: Display + Send + Sync + 'static {
        if let Crash::Error(e) = self {
            Crash::Error(e.context(context))
        } else {
            self
        }
    }

}
