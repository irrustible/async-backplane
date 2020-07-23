use async_channel::{self, Sender};
use concurrent_queue::ConcurrentQueue;
use intmap::IntMap;
use std::convert::TryInto; // turn a usize into a u64
use std::sync::atomic::{AtomicUsize, spin_loop_hint};
use rw_lease::{ReadGuard, Blocked, RWLease};

use crate::{DeviceID, Disconnect, Line, LinkError};
use crate::utils::BulkSend;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LineOp {
    Attach(Line),
    Detach(DeviceID),
}

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
    pub(crate) fn attach(&self, line: Line, error: LinkError) -> Result<(), LinkError> {
        self.lines.push(LineOp::Attach(line)).map_err(|_| error)
    }
    pub(crate) fn detach(&self, did: DeviceID, error: LinkError) -> Result<(), LinkError> {
        self.lines.push(LineOp::Detach(did)).map_err(|_| error)
    }
    /// Announce to all our monitors that we are disconnecting
    pub(crate) fn broadcast(&self, did: DeviceID, disconnect: Disconnect)
                            -> BulkSend<(DeviceID, Disconnect)>
    {
        // It may take a few moments to drain, so let's kick that off first.
        let mut drain = self.disconnects.try_write().unwrap();
        self.lines.close(); // no point taking any more link requests
        // Now we need to figure out which lines are mapped after
        // executing all the ops left for us
        let mut lines = IntMap::new();
        while let Ok(op) = self.lines.pop() {
            match op {
                LineOp::Attach(line) => {
                    lines.insert(line.device_id().inner.try_into().unwrap(), line);
                }
                LineOp::Detach(did) => {
                    lines.remove(did.inner.try_into().unwrap());
                }
            }
        }
        let mut bulk = BulkSend::new((did, disconnect));
        for (_, line) in lines.drain() {
            if let Some(sender) = line.plugboard.clone_sender() {
                bulk.add_sender(sender)
            }
        }
        // The readers have *probably* drained away by now
        loop { // Danger Will Robinson - will we spin forever?
            match drain.try_upgrade() {
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

    fn clone_sender(&self) -> Option<Sender<(DeviceID, Disconnect)>> {
        if let Ok(lock) = self.read_disconnects() {
            if let Some(sender) = &*lock { return Some(sender.clone()); }
        }
        None
    }

    // spin loop for read access
    fn read_disconnects<'a>(&'a self) ->
        Result<ReadGuard<'a, Option<Sender<(DeviceID, Disconnect)>>, AtomicUsize>, LinkError>
    {
        loop { // Danger Will Robinson - will we spin forever?
            match self.disconnects.try_read() {
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
