use crate::{Exit, Pid};
use futures::channel::mpsc::Sender;
use std::hash::{Hash, Hasher};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Link<Ret, Err>
where Ret: Clone, Err: Clone + Debug{
  pub pid: Pid,
  pub(crate) sender: Sender<Exit<Ret, Err>>,
}

impl<Ret, Err> Link<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  pub(crate) fn new(pid: Pid, sender: Sender<Exit<Ret, Err>>) -> Link<Ret, Err> {
    Link { pid, sender }
  }
}

impl<Ret, Err> Hash for Link<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.pid.hash(state);
  }
}

impl<Ret, Err> PartialEq for Link<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  fn eq(&self, other: &Self) -> bool {
    self.pid == other.pid
  }
}

impl<Ret, Err> Eq for Link<Ret, Err> where Ret: Clone, Err: Clone + Debug {}
