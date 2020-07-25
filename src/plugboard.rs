use async_channel::{self, Sender};
use concurrent_queue::ConcurrentQueue;
use std::convert::TryInto; // turn a usize into a u64
use std::sync::atomic::{AtomicUsize, spin_loop_hint};
use rw_lease::{ReadGuard, Blocked, RWLease};

use crate::{DeviceID, Disconnect, Line, LinkError};
use crate::linemap::{LineMap, LineOp};
use crate::utils::BulkSend;


#[derive(Debug)]
pub(crate) struct Plugboard {
    lines: ConcurrentQueue<LineOp>,
    disconnects: RWLease<Option<Sender<(DeviceID, Disconnect)>>>,
}

impl Plugboard {

    pub(crate) fn new(sender: Sender<(DeviceID, Disconnect)>) -> Self {
        let lines = ConcurrentQueue::unbounded();
        let disconnects = RWLease::new(Some(sender));
        Plugboard { lines, disconnects }
    }

    // Record that we wish to notify this Device when we disconnect.
    pub(crate) fn attach(&self, line: Line, error: LinkError) -> Result<(), LinkError> {
        self.lines.push(LineOp::Attach(line)).map_err(|_| error)
    }

    // Record that we no longer wish to notify this Device when we disconnect.
    pub(crate) fn detach(&self, did: DeviceID, error: LinkError) -> Result<(), LinkError> {
        self.lines.push(LineOp::Detach(did)).map_err(|_| error)
    }

    // Announce to all our monitors that we are disconnecting
    pub(crate) fn broadcast(&self, did: DeviceID, disconnect: Disconnect)
                            -> BulkSend<(DeviceID, Disconnect)>
    {
        // It may take a few moments to drain, so let's kick that off first.

        let mut drain = self.disconnects.write().unwrap();
        self.lines.close(); // no point taking any more link requests
        // Now we need to figure out which lines are mapped after
        // executing all the ops left for us
        let mut lines = LineMap::new();
        while let Ok(op) = self.lines.pop() {
            lines.apply(op);
        }
        let mut bulk = BulkSend::new((did, disconnect));
        for (_, maybe) in lines.drain() {
            if let Some(line) = maybe {
                if let Some(sender) = line.plugboard.clone_sender() {
                    bulk.add_sender(sender)
                }
            }
        }
        // The readers have *probably* drained away by now
        loop { // Danger Will Robinson - will we spin forever?
            match drain.upgrade() {
                Ok(mut sender_option) => {
                    #[allow(unused_must_use)]
                    sender_option.take();
                    return bulk;
                }
                Err(new_drain) => {
                    drain = new_drain;
                    spin_loop_hint();
                }
            }
        }
    }

    // This is the only place we read on the RWLease.
    fn clone_sender(&self) -> Option<Sender<(DeviceID, Disconnect)>> {
        if let Ok(lock) = self.read_disconnects() {
            if let Some(sender) = &*lock {
                return Some(sender.clone());
            }
        }
        None
    }

    // Attempt to obtain read access to the channel of disconnects
    fn read_disconnects<'a>(&'a self) ->
        Result<ReadGuard<'a, Option<Sender<(DeviceID, Disconnect)>>, AtomicUsize>, LinkError>
    {
        loop { // Eh, how many readers can there be?
            match self.disconnects.read() {
                // We're in, are we still alive?
                Ok(read) => { return Ok(read); }
                // The writer only locks to remove it
                Err(Blocked::Writer) => { return Err(LinkError::LinkDown); }
                // Or we could get dizzy...
                _ => { spin_loop_hint(); }
            }
        }
    }
}
