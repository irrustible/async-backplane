use core::fmt::{self, Display, Formatter};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::{Bad, Boson, Good, HANGUP, Name, Oops, Superposition};
use crate::promise::{Promise, Promised};
use piper::Sender;

/// A means of interacting with a Quantum
#[derive(Clone)]
pub struct Tangle<S: Superposition> {
  /// The quantum this tangle belongs to
  pub name: Name,
  sender: Sender<Boson<S>>,
  result: Promise<S>,
}

impl<S: Superposition> Unpin for Tangle<S> {}

impl<S: Superposition> Display for Tangle<S> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    self.name.fmt(f)
  }
}

impl<S: Superposition> Future for Tangle<S> {
  type Output = Result<S, Oops>;
  fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
    let mut this = self.as_mut();
    Promise::poll(Pin::new(&mut this.result), context)
      .map(|me| me.eat())
  }
}

impl<S: Superposition> Tangle<S> {
  pub(crate) fn new(
    name: Name,
    sender: Sender<Boson<S>>,
    result: Promise<S>
  ) -> Tangle<S> {
    Tangle { name, sender, result }
  }

  pub(crate) async fn send(&self, boson: Boson<S>) {
    self.sender.send(boson).await;
  }
}
