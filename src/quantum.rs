use core::convert::From;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use crate::{Bad, Boson, Entanglement, Good, Name, Oops, Superposition, Tangle};
use crate::backend;
use crate::promise::{Promise, WeakResolver};
use crate::observables::{Observable, Observables};
// use futures::Select;
use futures::stream::Stream;
use piper::{chan, Receiver, Sender};

pub struct Quantum<S: Superposition> {
  tangle: Tangle<S>,
  receiver: Receiver<Boson<S>>,
  resolver: WeakResolver<S>,
  observables: Observables<S>,
}

impl<S: Superposition> Quantum<S> {

  pub(crate) fn new(channel_size: usize) -> Quantum<S> {
    let name = Name::next();
    let (sender, receiver) = chan(8);
    let (resolver, promise) = WeakResolver::new_pair();
    let tangle = Tangle::new(name, sender, promise);
    let observables = Observables::new();
    Quantum { tangle, receiver, resolver, observables }
  }

  pub async fn entangle(&mut self, with: Tangle<S>, entanglement: Entanglement) {
    if let Entanglement::Entangled = entanglement {
      with.send(Boson::Entangle(self.tangle.clone())).await;
    }
    let observable = Observable::new(with, entanglement);
    self.observables.insert(observable);
  }

  pub async fn untangle(&mut self, from: Name) {
    if let Some(observable) = self.observables.remove(from) {
      if let Entanglement::Entangled = observable.entanglement {
        observable.tangle.send(Boson::Untangle(self.tangle.name)).await;
      }
    }
  }

  pub fn exit(&mut self, value: Result<S, Oops>) {
    self.resolver.resolve(value);
  }

  pub async fn request_exit(&mut self, who: Name, value: Result<S, Oops>) -> bool {
    if let Some(observable) = self.observables.get(who) {
      observable.tangle.send(Boson::ExitWith(value)).await;
      true
    } else {
      false
    }
  }
}

// impl<S: Superposition> Stream for Quantum<S> {
// }

pub enum Notification<T> {
  Entangled(Name),
  Untangled(Name),
  Observed(T),
}

