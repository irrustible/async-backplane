//! Utilities for dealing with panics (and unlocking better debugging).
//!
//! Important: be careful about installing panic handlers. Do it only
//! once per thread and pick your function carefully.
use futures_micro::poll_state;
use maybe_unwind::{capture_panic_info, maybe_unwind};
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::task::Poll;

pub use maybe_unwind::Unwind;

/// Sets the thread local panic handler to record the unwind information.
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

/// Run a future such that panics are converted into Unwinds.
pub async fn dont_panic<F, T>(future: F) -> Result<T, Unwind>
where F: Future<Output = T> {
    poll_state(Some(future), |future, ctx| {
        if let Some(ref mut fut) = future {
            let pin = unsafe { Pin::new_unchecked(fut) };
            match maybe_unwind(AssertUnwindSafe(|| <F as Future>::poll(pin, ctx))) {
                Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
                Err(unwind) => Poll::Ready(Err(unwind)),
                Ok(Poll::Pending) => Poll::Pending,
            }
        } else { Poll::Pending }
    }).await
}
