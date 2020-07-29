#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

pub mod utils;
pub mod panic;

mod plugboard;
mod device;
mod linemap;

pub use anyhow::{anyhow as error, bail as crash, ensure, Error};
pub use device::{Device, Line, Watched};

use maybe_unwind::Unwind;
use std::fmt::Display;

/// A locally unique identifier for a Device.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceID {
    pub(crate) inner: usize,
}

impl DeviceID {
    pub(crate) fn new(inner: usize) -> DeviceID {
        DeviceID { inner }
    }
}

/// Pairs a DeviceID with a result.
#[derive(Debug)]
pub struct Report<T> {
    pub device_id: DeviceID,
    pub result: T,
}

impl<T> Report<T> {
    pub fn new(device_id : DeviceID, result: T) -> Report<T> {
        Report { device_id, result }
    }
}

impl<T: Clone> Clone for Report<T> {
    fn clone(&self) -> Report<T> {
        Report {
            device_id: self.device_id,
            result: self.result.clone(),
        }
    }
}

impl<T: Copy> Copy for Report<T> {}

impl<T: PartialEq> PartialEq for Report<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.device_id == other.device_id) &&
            (self.result == other.result)
    }
}

impl<T: Eq> Eq for Report<T> {}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
/// There was a problem performing a Link.
pub enum LinkError {
    /// We can't because we are down.
    DeviceDown,
    /// We can't because the other Device is down.
    LinkDown,
}

/// The device has dropped off the backplane unexpectedly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Fault {
    /// Wasn't scheduled on an executor.
    Drop,
    /// The device errored.
    Error,
    /// A device we depended on errored.
    Cascade(DeviceID),
}

impl Fault {

    /// Whether the fault was a drop.
    pub fn is_drop(&self) -> bool { *self == Fault::Drop }

    /// Whether the fault was an error.
    pub fn is_error(&self) -> bool { *self == Fault::Error }

    /// Whether the disconnect was a cascade.
    pub fn is_cascade(&self) -> bool {
        if let Fault::Cascade(_) = self { true } else { false }
    }
    
}

#[derive(Copy, Clone)]
#[repr(u32)]
/// How deeply should we link?
pub enum LinkMode {
    /// Receive a notification when the other Device disconnects.
    Monitor = 0b01,
    /// Send a notification when we disconnect.
    Notify  = 0b10,
    /// Monitor + Notify.
    Peer    = 0b11,
}

impl LinkMode {
    /// true if we should be notified when the other Device disconnects.
    pub fn monitor(&self) -> bool {
        LinkMode::Monitor as u32 == ((*self) as u32 & LinkMode::Monitor as u32)
    }
    /// true if we should notify the other Device when we disconnect.
    pub fn notify(&self) -> bool {
        LinkMode::Notify as u32 == ((*self) as u32 & LinkMode::Notify as u32)
    }
    /// true if both sides will notify the other on disconnect.
    pub fn peer(&self) -> bool {
        LinkMode::Peer as u32 == ((*self) as u32 & LinkMode::Peer as u32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A message exchanged between devices.
pub enum Message {
    /// A Device we are monitoring has disconnected.
    Disconnected(Report<Option<Fault>>),
    /// Request to stop running.
    Shutdown(DeviceID),
}

pub use Message::{Disconnected, Shutdown};

impl Message {
    /// Unwraps the Disconnect notification or panics.
    pub fn unwrap_disconnected(self) -> Report<Option<Fault>> {
        if let Disconnected(report) = self { report }
        else { panic!("Message was not Disconnected") }
    }
}


/// Something went wrong with a Device.
#[derive(Debug)]
pub enum Crash<C=Error> {
    /// We were asked to shut down.
    PowerOff(DeviceID),
    /// The Device panicked.
    Panic(Unwind),
    /// The Device returned an Err.
    Error(C),
    /// A device we depend upon disconnected.
    Cascade(Report<Fault>),
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
        if let Crash::Cascade(_) = self {
            true
        } else {
            false
        }
    }

    /// Creates a disconnect representing this Crash.
    pub fn as_disconnect(&self) -> Option<Fault> {
        match self {
            Crash::PowerOff(_) => None,
            Crash::Panic(_) => Some(Fault::Error),
            Crash::Error(_) => Some(Fault::Error),
            Crash::Cascade(report) => Some(Fault::Cascade(report.device_id)),
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
