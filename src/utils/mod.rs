//! Futures helpers. Currently just `biased_race()` and `dont_panic()`.
mod dontpanic;
mod race;

pub use dontpanic::{dont_panic, DontPanic};
pub use maybe_unwind::Unwind;
pub use race::{biased_race, BiasedRace};
