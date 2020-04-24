#![feature(type_alias_impl_trait, async_closure)]
// generators, generator_trait, drain_filter, proc_macro_hygiene

mod backend;
use backend::*;

mod name;
pub use name::Name;

mod address;
pub use address::Address;

mod card;
pub use card::Card;

mod tangle;
pub use tangle::UntangleError;
pub use tangle::{Entangle, Tangle};

pub mod fewchores;

mod plugboard;
pub use plugboard::Plugboard;

pub struct HungUp {}

pub struct TimedOut {}

/// Whenever we are sending or receiving, we could either time out or hang up
pub enum AddressError {
  HungUp,
  TimedOut,
}

/// A request to exit
pub struct Exit<T: Clone> {
  pub from: Name,
  pub exit: T,
}

/// A notification that a process has exited
pub struct Exited<T: Clone> {
  pub name: Name,
  pub exit: T,
}

pub enum Message<T: Clone> {
  Entangle(Entangle<T>),
  Untangle(Name),
  Exit(Exit<T>),
  Exited(Exited<T>)
}

