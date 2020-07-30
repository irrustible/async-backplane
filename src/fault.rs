use crate::DeviceID;

/// The device has disconnected and it wasn't for a good reason.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Fault {
    /// Wasn't scheduled on an executor.
    Drop,
    /// Return an Err or panicked or generally something bad.
    Error,
    /// A device we depended on faulted.
    Cascade(DeviceID),
}

impl Fault {

    /// Did the Device drop without being scheduled?
    pub fn is_drop(&self) -> bool { *self == Fault::Drop }

    /// Did we return an Err or panic or something awful?
    pub fn is_error(&self) -> bool { *self == Fault::Error }

    /// Are we a cascade fault?
    pub fn is_cascade(&self) -> bool {
        if let Fault::Cascade(_) = self { true } else { false }
    }
    
}
