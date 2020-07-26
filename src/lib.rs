#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

pub mod utils;
pub mod panic;

mod plugboard;
mod device;
mod linemap;

pub use anyhow::{anyhow as error, bail as crash, ensure, Error};
pub use device::{Device, Line, Manage, PartManage, Watch};

use maybe_unwind::Unwind;
use std::fmt::Display;

/// Like 'Result', but with no semantic meaning
#[derive(Debug)]
pub enum Or<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Or<L, R> {
    pub fn is_left(&self) -> bool {
        if let Or::Left(_) = self { true } else { false }
    }
    pub fn is_right(&self) -> bool {
        if let Or::Right(_) = self { true } else { false }
    }
    pub fn unwrap_left(self) -> Option<L> {
        if let Or::Left(l) = self { Some(l) } else { None }
    }
    pub fn unwrap_right(self) -> Option<R> {
        if let Or::Right(r) = self { Some(r) } else { None }
    }
}

impl<L: PartialEq, R: PartialEq> PartialEq for Or<L, R> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Or::Left(l), Or::Left(r)) => l == r,
            (Or::Right(l), Or::Right(r)) => l == r,
            _ => false,
        }
    }
}

impl<L: Clone, R: Clone> Clone for Or<L, R> {
    fn clone(&self) -> Self {
        match self {
            Or::Left(l) => Or::Left(l.clone()),
            Or::Right(r) => Or::Right(r.clone()),
        }
    }
}

impl<L: Eq, R: Eq> Eq for Or<L, R> {}

/// A locally unique identifier for a Device
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceID {
    pub(crate) inner: usize,
}

impl DeviceID {
    pub(crate) fn new(inner: usize) -> DeviceID {
        DeviceID { inner }
    }
}

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
    /// We can't because we are down
    DeviceDown,
    /// We can't because the other Device is down
    LinkDown,
    /// We can't link to ourselves
    CantLinkSelf,
}

/// The device has dropped off the backplane unexpectedly
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Fault {
    /// Wasn't scheduled on an executor
    Drop,
    /// The device errored
    Error,
    /// A device we depended on errored
    Cascade(DeviceID),
}

impl Fault {

    /// Whether the disconnect was a crash.
    pub fn is_drop(&self) -> bool { *self == Fault::Drop }

    /// Whether the disconnect was a crash.
    pub fn is_error(&self) -> bool { *self == Fault::Error }

    /// Whether the disconnect was a cascade.
    pub fn is_cascade(&self) -> bool {
        if let Fault::Cascade(_) = self {
            true
        } else {
            false
        }
    }
    
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum LinkMode {
    Monitor = 0b01,
    Notify  = 0b10,
    Peer    = 0b11,
}

impl LinkMode {
    pub fn monitor(&self) -> bool {
        LinkMode::Monitor as u32 == ((*self) as u32 & LinkMode::Monitor as u32)
    }
    pub fn notify(&self) -> bool {
        LinkMode::Notify as u32 == ((*self) as u32 & LinkMode::Notify as u32)
    }
    pub fn peer(&self) -> bool {
        LinkMode::Peer as u32 == ((*self) as u32 & LinkMode::Peer as u32)
    }
}

type Disconnect = Report<Option<Fault>>;

/// Something went wrong with a Device
#[derive(Debug)]
pub enum Crash<C=Error> {
    /// The Device panicked.
    Panic(Unwind),
    /// The Device returned an Err
    Error(C),
    /// A device we depend upon disconnected
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
    pub fn as_disconnect(&self) -> Fault {
        match self {
            Crash::Panic(_) => Fault::Error,
            Crash::Error(_) => Fault::Error,
            Crash::Cascade(report) => Fault::Cascade(report.device_id),
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
