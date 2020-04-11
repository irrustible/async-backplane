use crate::Pid;
use futures::Future;
use futures::channel::{mpsc, oneshot};
use futures::sink::SinkExt;
use std::time::Duration;

pub struct LinkRequest<T: Clone> {
  pid: Pid,
  ack: oneshot::Sender<()>,
  notify: mpsc::Sender<T>,
}

impl<T: Clone> LinkRequest<T> {
  pub fn new(pid: Pid, ack: oneshot::Sender<()>, notify: mpsc::Sender<T>) -> Self {
    LinkRequest { pid, ack, notify }
  }
}

impl<T: Clone> PartialEq for LinkRequest<T> {
  fn eq(&self, other: &Self) -> bool {
    self.pid == other.pid
  }
}

pub enum ControlMessage<T: Clone> {
  /// A request to link
  Link(LinkRequest<T>),
  /// A request to give up
  Cancel(T),
}

/// A Control handle allows you to link to or cancel a process
pub struct ControlHandle<T: Clone> {
  pub pid: Pid,
  handle: mpsc::Sender<ControlMessage<T>>,
}

// impl<T: Clone> ControlHandle<T> {
//   pub async fn request_link(&mut self, req: RequestLink<T>) -> (oneshot::Receiver<()>, impl Future) {
//     let sender = self.handle.send(ControlMessage::Link(req));
//     (ack_recv, sender)
//   }
//   pub fn request_cancel(&mut self, error: T) -> impl Future {
//     self.handle.send(ControlMessage::Cancel(error))
//   }
// }
