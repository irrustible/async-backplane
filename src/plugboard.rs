use concurrent_queue::ConcurrentQueue;
use crate::{DeviceID, Disconnect, Line, LinkError};
use crate::linemap::LineOp;
use waker_queue::WakerQueue;

#[derive(Debug)]
pub(crate) struct Plugboard {
    pub line_ops: ConcurrentQueue<LineOp>,
    pub disconnects: WakerQueue<Disconnect>,
}

impl Plugboard {

    pub fn new() -> Self {
        Plugboard {
            line_ops: ConcurrentQueue::unbounded(),
            disconnects: WakerQueue::unbounded(),
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

    // Notify this Device that we're done.
    pub fn notify(&self, report: Disconnect) -> Result<(), Disconnect> {
        self.disconnects.try_push_wake(report, true).map_err(|e| e.into_inner())
    }

    // Stop taking requests
    pub fn close(&self) {
        self.line_ops.close();
        self.disconnects.close();
    } 

}
