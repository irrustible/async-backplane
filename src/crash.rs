use crate::{DeviceID, Fault, panic::Unwind};

/// Something went wrong with a Device.
#[derive(Debug)]
pub enum Crash<Error> {
    /// We were asked to shut down.
    PowerOff(DeviceID),
    /// The Future we were executing panicked.
    Panic(Unwind),
    /// The Future we were executing returned an Err.
    Error(Error),
    /// A device we depended upon faulted.
    Cascade(DeviceID, Fault),
}

impl<Error> Crash<Error> {

    /// Did the future unwind panic?
    pub fn is_panic(&self) -> bool { matches!(self, Crash::Panic(_)) }

    /// Did the future return Err?
    pub fn is_error(&self) -> bool { matches!(self, Crash::Error(_)) }

    /// Did a Device we depend on fault?
    pub fn is_cascade(&self) -> bool { matches!(self, Crash::Cascade(_, _)) }

}
