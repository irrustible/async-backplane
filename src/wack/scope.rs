//! Taken from https://github.com/stjepang/piper/blob/master/src/event.rs, MIT/Apache2
//! A synchronization primitive for notifying async tasks and threads.
//!
//! This is a variant of a conditional variable that is heavily inspired by eventcounts invented
//! by Dmitry Vyukov: http://www.1024cores.net/home/lock-free-algorithms/eventcounts

use async_task::waker_fn;
use core::hash::Hash;
use im::Vector;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::collections::HashSet;

pub trait Scopeable : Clone + Eq + Hash + Send + Sync + 'static {}

impl<T: Clone + Eq + Hash + Send + Sync + 'static> Scopeable for T {}

#[derive(Clone)]
pub struct Scope<T: Scopeable> {
  inner: Arc<Mutex<HashSet<T>>>,
}

impl<T: Scopeable> Scope<T> {
  pub(crate) fn new() -> Self {
    Scope { inner: Arc::new(Mutex::new(HashSet::new())) }
  }
  pub(crate) fn drain(&self) -> Vector<T> {
    let mut data = self.inner.lock().unwrap();
    (*data).drain().collect()
  }
}

// This is funky. We shouldn't need to clone. But the entire thing is
// terrible at the moment anyway so who cares?
pub fn scoped<T: Scopeable>(scope: Scope<T>, key: T) -> Waker {
  waker_fn(move || {
    let mut data = scope.inner.lock().unwrap();
    (*data).insert(key.clone());
  })
}
