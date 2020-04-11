use crate::{Exit, Link};
use futures::Future;
use futures::channel::mpsc::Sender;
use futures::sink::SinkExt;
use futures::stream::FuturesUnordered;
use std::collections::HashSet;
use std::fmt::Debug;

/**
 * links is responsible for maintaining a collection of processes
 * interested in the fate of this process and broadcasting to them
 */

#[derive(Clone)]
pub struct Links<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  links: HashSet<Link<Ret, Err>>,
}

impl<Ret, Err> Links<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  pub fn new() -> Links<Ret, Err> {
    Links { links: HashSet::new() }
  }

  pub fn link(&mut self, l: Link<Ret, Err>) -> bool {
    self.links.insert(l)
  }

  pub fn broadcast(self, message: Exit<Ret, Err>) -> FuturesUnordered<impl Future<Output=()>> {
    self.links.into_iter().map(|mut link| {
      let m = message.clone();
      async move {
        if let Err(e) = link.sender.send(m).await {
          if e.is_full() { panic!("Who ate all my channel?") }
        };
      }
    }).collect()
  }
  
}
