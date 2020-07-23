//! Futures helpers. Currently just `biased_race()` and `dont_panic()`.
mod race;
mod sending;
mod dontpanic;

pub use race::{biased_race, BiasedRace};
pub(crate) use sending::BulkSend;
pub use dontpanic::{dont_panic, DontPanic};
