use std::pin::Pin;
use core::convert::From;
use core::future::Future;
use core::task::{Context, Poll};
use futures::stream::{FuturesUnordered, Stream};

pub struct FuturesEndless<F: Future> {
  inner: FuturesUnordered<F>,
}

impl<F: Future> FuturesEndless<F> {

  pub fn new() -> Self {
   FuturesEndless { inner: FuturesUnordered::new() }
  }

  pub fn push(&mut self, future: F) {
    self.inner.push(future);
  }
}

impl<F: Future> From<FuturesUnordered<F>> for FuturesEndless<F> {
  fn from(inner: FuturesUnordered<F>) -> Self {
    FuturesEndless { inner }
  }
}

impl<F: Future> Stream for FuturesEndless<F> {
  type Item = F::Output;

  fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
    if self.inner.is_empty() {
      Poll::Pending
    } else {
      FuturesUnordered::poll_next(Pin::new(&mut self.get_mut().inner), context)
    }
  }
}
