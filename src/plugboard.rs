use crate::{Address, AddressError, Card, Name, Exit, Exited, Message, Tangle, UntangleError};
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use futures::stream::Stream;
use piper::Receiver;

pub struct Plugboard<T: Clone> {
  receiver: Receiver<Message<T>>,
  tangle: Tangle<T>,
}

impl<T: Clone> Unpin for Plugboard<T> {}

impl<T: Clone> Plugboard<T> {

  pub fn card(&self) -> &Card<T> {
    &self.tangle.card
  }

  pub fn name(&self) -> &Name {
    &self.tangle.card.name
  }

  pub fn address(&self) -> &Address<T> {
    &self.tangle.card.address
  }

  pub async fn entangle(&mut self, with: Address<T>, timeout: Duration) ->
    Result<Name, AddressError> {
      self.tangle.entangle(with, timeout).await
  }
  
  pub async fn untangle(&mut self, from: Name, timeout: Duration) ->
    Result<(), UntangleError> {
      self.tangle.untangle(from, timeout).await
  }

  #[allow(unused_must_use)]
  pub async fn exit(self, value: T, timeout: Duration) {
    self.tangle.exit(value, timeout).await;
  }
    
}

impl<T: Clone> Stream for Plugboard<T> {

  type Item = Message<T>;

  fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
    Receiver::poll_next(Pin::new(&mut self.as_mut().receiver), context)
  }

}
