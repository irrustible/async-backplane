use core::fmt;
pub use core::convert::From;

/// A locally unique identifier for a Device.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceID {
    pub(crate) inner: usize,
}

impl DeviceID {
    pub(crate) fn new(inner: usize) -> DeviceID {
        DeviceID { inner }
    }
}

impl From<DeviceID> for usize {
    fn from(did: DeviceID) -> usize { did.inner }
}

impl fmt::Debug for DeviceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("DeviceID<{:x}>", self.inner))
    }
}

impl fmt::Display for DeviceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("DeviceID<{:x}>", self.inner))
    }
}
