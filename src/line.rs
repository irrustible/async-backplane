use crate::plugboard::Plugboard;
use crate::*;
use core::fmt;
use std::sync::Arc;

/// A reference to a `Device` that allows us to link with it.
#[derive(Clone)]
pub struct Line {
    pub(crate) plugboard: Arc<Plugboard>,
}

impl Line {
    /// Get the ID of the Device this line is connected to.
    pub fn device_id(&self) -> DeviceID {
        DeviceID::new(&*self.plugboard as *const _ as usize)
    }

    /// Send a message to the Device. Returns the original message on
    /// failure (if the Device has disconnected).
    pub fn send(self, message: Message) -> Result<(), Message> {
        self.plugboard.send(message)
    }

    /// Links with a Device through its Line. Panics if you try to link to yourself.
    pub fn link_line(&self, other: Line, mode: LinkMode) -> Result<(), LinkError> {
        if self.device_id() == other.device_id() {
            panic!("Do not link to yourself.");
        }
        if mode.monitor() {
            other.plugboard.plug(self.clone(), LinkError::LinkDown)?;
        }
        if mode.notify() {
            self.plugboard.plug(other, LinkError::DeviceDown)?;
        }
        Ok(())
    }

    /// Links with another Line.
    pub fn unlink_line(&self, other: &Line, mode: LinkMode) {
        #[allow(unused_must_use)]
        if self.device_id() != other.device_id() {
            if mode.monitor() {
                other
                    .plugboard
                    .unplug(self.device_id(), LinkError::LinkDown);
            }
            if mode.notify() {
                self.plugboard
                    .unplug(other.device_id(), LinkError::DeviceDown);
            }
        }
    }
}

impl Eq for Line {}

impl Unpin for Line {}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        Arc::ptr_eq(&self.plugboard, &other.plugboard)
    }
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Line<{:x}>", self.device_id().inner))
    }
}
