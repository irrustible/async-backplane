use async_channel::{self, Sender};
use concurrent_queue::ConcurrentQueue;
use intmap::IntMap;
use smol;
use std::convert::TryInto; // turn a usize into a u64
use std::sync::atomic::{AtomicUsize, spin_loop_hint};
use rw_lease::{ReadGuard, Blocked, RWLease};

use crate::{DeviceID, Disconnect, Line, LinkError, Pluggable};

#[derive(Clone, Eq, PartialEq)]
pub enum LineOp {
    Attach(Line),
    Detach(DeviceID),
}

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
    pub(crate) fn broadcast(&self, did: DeviceID, disconnect: Disconnect) {
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
        for (_, line) in lines.drain() {
            line.plugboard.notify(did, disconnect);
        }
        // The readers have *probably* drained away by now
        loop { // Danger Will Robinson - will we spin forever?
            match drain.try_upgrade() {
                Ok(mut result) => {
                    #[allow(unused_must_use)]
                    result.take();
                    return;
                }
                Err(new_drain) => {
                    drain = new_drain;
                    spin_loop_hint();
                }
            }
        }
    }

    /// Announce our exit to the device this plugboard belongs to
    pub(crate) fn notify(&self, did: DeviceID, disconnect: Disconnect) {
        if let Ok(lock) = self.read_disconnects() {
            if let Some(sender) = &*lock {
                let send = send_in_new_thread(sender.clone(), (did, disconnect));
                smol::Task::spawn(send).detach(); // expensive? also naughty - feature flag!
            }
        }
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

#[allow(unused_must_use)]
async fn send_in_new_thread<T: 'static + Send>(sender: Sender<T>, val: T) {
    sender.send(val).await;
    ()
}
