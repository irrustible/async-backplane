use atomic_waker::AtomicWaker;
use concurrent_queue::ConcurrentQueue;
use crate::{DeviceID, Disconnect, Line, LinkError};
use crate::linemap::LineOp;

#[derive(Debug)]
pub(crate) struct Plugboard {
    pub line_ops: ConcurrentQueue<LineOp>,
    pub disconnects: ConcurrentQueue<Disconnect>,
    pub waker: AtomicWaker,
}

impl Plugboard {

    pub fn new() -> Self {
        Plugboard {
            line_ops: ConcurrentQueue::unbounded(),
            disconnects: ConcurrentQueue::unbounded(),
            waker: AtomicWaker::new(),
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

    // Notify the Device whose plugboard this is that we're done.
    pub fn notify(&self, report: Disconnect) -> Result<(), Disconnect> {
        match self.disconnects.push(report) {
            Ok(()) => {
                self.waker.wake();
                Ok(())
            }
            Err(e) => Err(e.into_inner()),
        }
    }

    // Stop taking requests
    pub fn close(&self) {
        self.waker.take();
        self.line_ops.close();
        self.disconnects.close();
    }

}
