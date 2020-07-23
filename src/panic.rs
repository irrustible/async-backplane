//! Panic handler installer functions for better debugging.
//!
//! Call only *one* of these, only once per thread.
use std::panic;
use maybe_unwind::capture_panic_info;

/// Sets the thread local panic handler to record the unwind information
pub fn replace_panic_hook() {
    panic::set_hook(Box::new(|info| { capture_panic_info(info); }));
}

/// Sets the thread local panic handler to record the unwind information
/// and then execute the existing hook.
pub fn chain_panic_hook() {
    let old = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        capture_panic_info(info);
        old(info);
    }));
}
