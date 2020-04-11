use crate::{Exit, Link, Pid};
use futures::{Future, Stream};
use futures::channel::oneshot;
use futures::sink::SinkExt;
use futures::stream::FuturesUnordered;
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::collections::HashSet;
use std::pin::Pin;

/**
 * Linking is responsible for polling for the acceptance of links we
 * have requested and applying a timeout (soon...)
 */

#[pin_project]
pub struct Linking<F>
where F: Future<Output=Result<Pid, Pid>> {
  #[pin]
  pending: FuturesUnordered<F>,
}

impl<F> Linking<F>
where F: Future<Output=Result<Pid, Pid>> {

  pub fn new() -> Linking<F> {
    Linking { pending: FuturesUnordered::new() }
  }

  // we can't do this because we can't write the type of the future :/
  pub fn join(&mut self, pid: Pid, receiver: oneshot::Receiver<()>) {
    self.pending.push(async move {
      let match receiver.await {
        Ok(_) => Ok(pid),
        Err(_) => Err(pid),
      }
    })
  }
}

impl<F> Stream for Linking<F>
where F: Future<Output=Result<Pid, Pid>> {
  type Item = Result<Pid, Pid>;
  
  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Result<Pid, Pid>>> {
    let this = self.project();
    if this.pending.is_empty() {
      Poll::Pending
    } else {
      FuturesUnordered::poll_next(this.pending, cx)
    }
  }

}
