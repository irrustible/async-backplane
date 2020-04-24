use crate::Message;
use piper::Sender;

/// An address is where you can send a message
#[derive(Clone)]
pub struct Address<T: Clone> {
  sender: Sender<Message<T>>,
}

impl<T: Clone> Unpin for Address<T> {}

impl<T: Clone> Address<T> {

  pub fn is_empty(&self) -> bool {
    self.sender.is_empty()
  }

  pub fn is_full(&self) -> bool {
    self.sender.is_full()
  }

  pub fn len(&self) -> usize {
    self.sender.len()
  }

  pub(crate) async fn send(&self, msg: Message<T>) {
    self.sender.send(msg).await
  }

}
