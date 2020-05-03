use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::{Bad, Entanglement, Good, Name, Oops, Superposition, Tangle};
use crate::wack::{scoped, Scope};
use futures::stream::Stream;
use intmap::IntMap;
use im::Vector;

pub struct Observable<S: Superposition> {
  pub tangle: Tangle<S>,
  pub entanglement: Entanglement,
}

impl<S: Superposition> Unpin for Observable<S> {}

impl<S: Superposition> Future for Observable<S> {
  type Output = Result<S, Oops>;

  fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
    let mut this = self.as_mut();
    Tangle::poll(Pin::new(&mut this.tangle), context)
  }
}

impl<S: Superposition> Observable<S> {
  pub(crate) fn new(tangle: Tangle<S>, entanglement: Entanglement) -> Observable<S> {
    Observable { tangle, entanglement }
  }

}

/// A collection of quanta whose exits we care aboutn
pub struct Observables<S: Superposition> {
  inner: IntMap<Observable<S>>,
  scope: Scope<Name>,
  pending: Vector<Name>,
}

impl<S: Superposition> Observables<S> {

  pub fn new() -> Self {
    Observables {
      inner: IntMap::new(),
      scope: Scope::new(),
      pending: Vector::new(),
    }
  }

  pub fn get(&self, name: Name) -> Option<&Observable<S>> {
    self.inner.get(name.inner)
  }

  pub fn insert(&mut self, observable: Observable<S>) {
    self.inner.insert(observable.tangle.name.inner, observable);
  }

  pub fn remove(&mut self, name: Name) -> Option<Observable<S>> {
    self.inner.remove(name.inner)
  }

  fn poll_pending(&mut self, context: &mut Context) -> Option<Result<S, Oops>> {
    while let Some(pending) = self.pending.pop_front() {
      if let Some(mut obs) = self.inner.remove(pending.inner) {
        if let Poll::Ready(poll) = Observable::poll(Pin::new(&mut obs), context) {
          return Some(poll);
        } else {
          self.inner.insert(pending.inner, obs);
        }
      }
    }
    None
  }
}

impl<S: Superposition> Stream for Observables<S> {
  type Item = Result<S, Oops>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    context: &mut Context
  ) -> Poll<Option<Self::Item>> {
    let mut this = self.as_mut();
    // First we check the pending pile
    match this.poll_pending(context) {
      Some(item) => Poll::Ready(Some(item)),
      None => {
        this.pending = this.scope.drain();
        match this.poll_pending(context) {
          Some(item) => Poll::Ready(Some(item)),
          None => Poll::Pending,
        }
      }
    }
  }
}
