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
    pub fn is_panic(&self) -> bool {
        if let Crash::Panic(_) = self { true } else { false }
    }

    /// Did the future return Err?
    pub fn is_error(&self) -> bool {
        if let Crash::Error(_) = self { true } else { false }
    }

    /// Did a Device we depend on fault?
    pub fn is_cascade(&self) -> bool {
        if let Crash::Cascade(_, _) = self { true } else { false }
    }

}
