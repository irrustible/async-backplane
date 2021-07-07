use crate::linemap::LineOp;
use crate::{DeviceID, Line, LinkError, Message};
use concurrent_queue::ConcurrentQueue;
use waker_queue::WakerQueue;

#[derive(Debug)]
pub(crate) struct Plugboard {
    pub line_ops: ConcurrentQueue<LineOp>,
    pub messages: WakerQueue<Message>,
}

impl Plugboard {
    pub fn new() -> Self {
        Plugboard {
            line_ops: ConcurrentQueue::unbounded(),
            messages: WakerQueue::unbounded(),
        }
    }

    // Record that we wish to notify this Device when we disconnect.
    pub fn plug(&self, line: Line, error: LinkError) -> Result<(), LinkError> {
        self.line_ops.push(LineOp::Attach(line)).map_err(|_| error)
    }

    // Record that we no longer wish to notify this Device when we disconnect.
    pub fn unplug(&self, did: DeviceID, error: LinkError) -> Result<(), LinkError> {
        self.line_ops.push(LineOp::Detach(did)).map_err(|_| error)
    }

    // Send a message down the line.
    pub fn send(&self, message: Message) -> Result<(), Message> {
        self.messages
            .try_push_wake(message, true)
            .map_err(|e| e.into_inner())
    }

    // Stop taking requests
    pub fn close(&self) {
        self.line_ops.close();
        self.messages.close();
    }
}
