mod race;
mod sending;
mod dontpanic;

pub use race::{biased_race, BiasedRace};
pub(crate) use sending::BulkSend;
pub use dontpanic::DontPanic;

use std::panic;
use maybe_unwind::capture_panic_info;

/// Sets the thread local panic handler to record the unwind information
pub fn replace_panic_hook() {
    panic::set_hook(Box::new(|info| { capture_panic_info(info); }));
}

/// Sets the thread local panic handler to record the unwind information
/// and then execute whichever other hook was already in place
pub fn chain_panic_hook() {
    let old = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        capture_panic_info(info);
        old(info);
    }));
}
