pub mod panic;
pub mod prelude;
pub mod utils;

mod crash;
pub use crash::Crash;

mod device_id;
pub use device_id::DeviceID;

mod device;
pub use device::Device;

mod fault;
pub use fault::Fault;

mod line;
pub use line::Line;

mod watched;
pub use watched::Watched;

mod linemap;
mod plugboard;

// These are small, here will be fine.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// There was a problem Linking
pub enum LinkError {
    /// We can't because we already disconnected.
    DeviceDown,
    /// We can't because the other Device already disconnected.
    LinkDown,
}

#[derive(Clone, Copy)]
#[repr(u32)]
/// How deeply should we link?
pub enum LinkMode {
    /// Receive a notification when the other Device disconnects.
    Monitor = 0b01,
    /// Send a notification when we disconnect.
    Notify = 0b10,
    /// Monitor + Notify.
    Peer = 0b11,
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
    Disconnected(DeviceID, Option<Fault>),
    /// Request to stop running.
    Shutdown(DeviceID),
}

use Message::{Disconnected, Shutdown};

impl Message {
    /// Returns the DeviceID of the sender.
    pub fn sender(&self) -> DeviceID {
        match self {
            Disconnected(did, _) => *did,
            Shutdown(did) => *did,
        }
    }

    /// Unwraps the Disconnect notification or panics.
    pub fn unwrap_disconnected(&self) -> (DeviceID, Option<Fault>) {
        if let Disconnected(did, fault) = self {
            (*did, *fault)
        } else {
            panic!("Message was not Disconnected")
        }
    }

    /// Unwraps the Shutdown request or panics.
    pub fn unwrap_shutdown(&self) -> DeviceID {
        if let Shutdown(did) = self {
            *did
        } else {
            panic!("Message was not Shutdown")
        }
    }
}
