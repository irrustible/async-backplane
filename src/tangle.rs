use crate::fewchores::FewchoresExt;
use crate::{Address, AddressError, Card, Exited, Message, Name};
use futures::stream::{FuturesUnordered, StreamExt};
use intmap::IntMap;
use piper::{chan, Sender, Receiver};
use smol::Timer;
use std::iter;
use std::time::Duration;

pub struct Tangle<T: Clone> {
  pub card: Card<T>,
  tangled: IntMap<Address<T>>,
}

/// Request to link
pub struct Entangle<T: Clone> {
  card: Card<T>,
  reply: Sender<Result<Address<T>, ()>>,
}

/// Intermediate link state
struct Pending<T: Clone> {
  name: Name,
  receiver: Receiver<Result<Address<T>, ()>>,
}

pub enum UntangleError {
  NotTangled,
  TimedOut,
}

impl<T: Clone> Entangle<T> {

  fn new_pair(card: Card<T>) -> (Entangle<T>, Pending<T>) {
    let (reply, receiver) = chan(1);
    let wait = Pending { receiver, name: card.name };
    (Entangle { card, reply }, wait)
  }

}

impl<T: Clone> Pending<T> {

  async fn wait(self) -> Result<Result<Card<T>, Name>, AddressError> {
    match self.receiver.recv().await {
      Some(Ok(address)) => Ok(Ok(Card::new(self.name, address))),
      Some(Err(())) => Ok(Err(self.name)),
      None => Err(AddressError::HungUp),
    }
  }

}

impl<T: Clone> Unpin for Tangle<T> {}

impl<T: Clone> Tangle<T> {

  /// Initiate linking with the provided address, timing out after the
  /// provided Duration
  pub async fn entangle(&mut self, to: Address<T>, timeout: Duration) -> Result<Name, AddressError> {
    let (entangle, pending) = Entangle::new_pair(self.card.clone());
    match self.do_entangle(to, entangle, pending).race(Timer::after(timeout)).await {
      Ok(Ok(ok)) => Ok(ok),
      Ok(Err(e)) => Err(e),
      Err(_) => Err(AddressError::TimedOut),
    }
  }

  // used by entangle()
  async fn do_entangle(&mut self, to: Address<T>, entangle: Entangle<T>, pending: Pending<T>) -> Result<Name, AddressError> {
    to.send(Message::Entangle(entangle)).await;
    match pending.wait().await {
      Ok(Ok(card)) => {
        self.tangled.insert(card.name.inner, card.address);
        Ok(card.name)
      }
      Ok(Err(name)) => Ok(name),
      Err(err) => Err(err),
    }
  }

  /// Initiate unentangleing with the provided name, timing out after the
  /// provided Duration
  pub async fn untangle(&mut self, from: Name, timeout: Duration) -> Result<(), UntangleError> {
    if let Some(address) = self.tangled.remove(from.inner) {
      address.send(Message::Untangle(self.card.name))
        .race(Timer::after(timeout)).await
        .map_err(|_| UntangleError::TimedOut)
    } else {
      Err(UntangleError::NotTangled)
    }
  }

  /// exit, broadcasting the provided exit message to everybody
  pub async fn exit(mut self, exit: T, timeout: Duration) {
    let name = self.card.name;
    let stream = FuturesUnordered::new();
    for ((_key, address), exit) in self.tangled.drain().zip(iter::repeat(exit)) {
      stream.push(async move {
        address.send(Message::Exited(Exited { name, exit })).await;
      });
    }
    stream.fold((), |_, _| async {}).race(Timer::after(timeout)).await;
  }

  /// Call me when a link request comes in
  pub async fn accept_entangle(&mut self, entangle: Entangle<T>) -> Result<Name, ()> {
    let name = entangle.card.name;
    if self.tangled.get(name.inner).is_none() {
      entangle.reply.send(Ok(self.card.address.clone())).await;
      Ok(name)
    } else {
      entangle.reply.send(Err(())).await;
      Err(())
    }
  }

  /// Call me when an unentangle request comes in
  pub fn accept_untangle(&mut self, name: Name) -> Result<(), UntangleError> {
    if self.tangled.remove(name.inner).is_some() { Ok(()) }
    else { Err(UntangleError::NotTangled) }
  }

}

