#![feature(type_alias_impl_trait, min_specialization)] // async_closure, trait_alias

mod wack;

pub mod fewchores;

pub mod promise;
use promise::{Promise, WeakResolver};

mod backend;

mod name;
pub use name::Name;

mod tangle;
pub use tangle::Tangle;

mod observables;

mod quantum;
// use async_task::waker_fn;
use core::fmt::{self, Display, Formatter};
use core::future::Future;
use core::time::Duration;
use maybe_unwind::Unwind;
use piper::{Receiver, Sender};
use std::error;
use std::sync::Arc;
use thiserror::Error;

/// We do not yet know which way the value resolved
pub trait Superposition : Clone + 'static {
  type Success : Good;
  type Failure : Bad;
  fn succeeded(self) -> bool;
  fn failed(self) -> bool;
}

impl<G: Good, B: Bad> Superposition for Result<G, B> {
  type Success = G;
  type Failure = B;
  fn succeeded(self) -> bool { self.is_ok() }
  fn failed(self) -> bool { self.is_err() }
}

/// A success value must satisfy me
pub trait Good : Clone + Display + 'static {}

impl<G: Clone + Display + 'static> Good for G {}

/// An error value must satisfy me
pub trait Bad : Clone + error::Error + Display + 'static {}

impl<B: Clone + std::error::Error + Display + 'static> Bad for B {}

#[derive(Eq, PartialEq)]
pub enum Entanglement {
  /// We're not sure the Observer collapses the wave function, but it
  /// will observe the other Quantum's Oops
  Observer,
  /// Additionally request the other Quantum to entangle with us so
  /// that it will also observe our Oops
  Entangled,
}

pub struct Spec {
  kind: Kind,
  shutdown: Shutdown,
  restart: Restart,
  restarts_per_period: usize,
  restart_period: Duration,
}

/// What kind of Quantum is this?
pub enum Kind {
  Normal,
  Supervisor,
}

/// How long should a supervisor give a Quantum to clean up when it
/// needs to be shut down?
pub enum Shutdown {
  DontWait,
  WaitFor(Duration),
  WaitIndefinitely,
}

/// Should the supervisor restart the Quantum if it terminates?
pub enum Restart {
  Always,
  OnError,
  Never,
}

#[derive(Clone, Debug, Error)]
#[error("hung up")]
pub struct Hangup {}

const HANGUP : Hangup = Hangup {};

#[derive(Clone, Debug, Error)]
#[error("panicked: {unwind}")]
pub struct Panic {
  pub unwind: Arc<Unwind>,
}

impl From<Unwind> for Panic {
  fn from(unwind: Unwind) -> Self {
    Panic { unwind: Arc::new(unwind) }
  }
}

/// It's an error in your code, Bob.
#[derive(Clone, Debug, Error)]
pub enum Oops {
  /// Got dropped, wasn't polled to completion.
  #[error(transparent)]
  Hangup(#[from] Hangup),
  /// Panicked, we can get you a stacktrace if you followed the README
  #[error(transparent)]
  Panic(#[from] Panic),
}

/// A measurement paired with a name
pub struct Observation<T> {
  pub name: Name,
  pub result: Result<T, Oops>,
}

impl<S: Superposition> Observation<S> {
  pub fn new(name: Name, result: Result<S, Oops>) -> Self {
    Observation { name, result }
  }
}

impl<S: Superposition + Display> Display for Observation<S> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match &self.result {
      Ok(val) => f.write_fmt(format_args!("{} decohered successfully: {}", self.name, val)),
      Err(val) => f.write_fmt(format_args!("{} {}", self.name, val)),
    }
  }
}

/// A 'force particle', a message exchanged between quanta.
enum Boson<S: Superposition> {
  Entangle(Tangle<S>),
  Untangle(Name),
  ExitWith(Result<S, Oops>),
}

