use futures::future::Future;
use futures::task::{Context, Poll};
use std::pin::Pin;

/// Race is like `futures::future::Select` but doesn't require Unpin of its contents.
pub struct Race<L: Future, R: Future> {
  l: L,
  r: R,
}

impl <L: Future + Unpin, R: Future + Unpin> Unpin for Race<L, R> {}

impl<L: Future, R: Future> Race<L, R> {
  pub fn new(l: L, r: R) -> Self {
    Race { l, r }
  }
}

impl<L: Future, R: Future> Future for Race<L, R> {
  type Output = Result<L::Output, R::Output>;
  fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
    let this = unsafe { self.get_unchecked_mut() };
    let pin = unsafe { Pin::new_unchecked(&mut this.l) };
    match L::poll(pin, context) {
      Poll::Ready(val) => Poll::Ready(Ok(val)),
      Poll::Pending => {
        let pin = unsafe { Pin::new_unchecked(&mut this.r) };
        match R::poll(pin, context) {
          Poll::Ready(val) => Poll::Ready(Err(val)),
          Poll::Pending => Poll::Pending,
        }
      }
    }
  }
}

