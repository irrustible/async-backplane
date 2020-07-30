//! Futures helpers. Currently just `biased_race()` and `dont_panic()`.
mod race;
mod dontpanic;

pub use dontpanic::{dont_panic, DontPanic};
pub use race::{biased_race, BiasedRace};
pub use maybe_unwind::Unwind;
