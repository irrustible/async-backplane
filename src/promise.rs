use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use crossbeam_utils::CachePadded;
use crate::{HANGUP, Oops};
use piper::{Event, EventListener};
use std::sync::{Arc, Weak};

/// A reader Future for a value computed concurrently. Logically an
/// `Arc<T>` which might not be set yet but which can be awaited.
pub struct Promise<T> {
  delayed: Option<Arc<Delayed<T>>>,
  event: Option<EventListener>,
}

impl<T> Unpin for Promise<T> {}

impl<T> Clone for Promise<T> {
  fn clone(&self) -> Self {
    Promise { delayed: self.delayed.clone(), event: None }
  }
}

impl<T> Future for Promise<T> {
  type Output = Promised<T>;
  
  fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Promised<T>> {
    if self.delayed.is_none() { panic!("Promise polled after completion") };
    let mut this = self.as_mut();
    loop {
      if let Some(ref mut event) = &mut this.event {
        if EventListener::poll(Pin::new(event), context).is_ready() { this.event = None }
        else { return Poll::Pending }
      } else {
        if this.delayed.as_ref().unwrap().state.load(Ordering::Acquire) {
          return Poll::Ready(Promised { inner: this.delayed.take().unwrap() });
        } else {
          this.event = Some(this.delayed.as_ref().unwrap().event.listen());
        }
      }
    }
  }
}

/// A handle for resolving a Promise
pub struct Resolver<T> {
  delayed: Option<Arc<Delayed<T>>>,
}

impl<T> Resolver<T> {

  /// Creates a new Promise and its corresponding Resolver
  pub fn new_pair() -> (Resolver<T>, Promise<T>) {
    let state = CachePadded::new(AtomicBool::new(false));
    let delayed = Arc::new(Delayed { state, data: UnsafeCell::new(None), event: Event::new() });
    (Resolver { delayed: Some(delayed.clone()) },
     Promise { delayed: Some(delayed), event: None })
  }

  /// Resolves for linked promises, notifying any that are polling
  pub fn resolve(&mut self, value: Result<T, Oops>) {
    self.delayed.take().map(|x| x.resolve(value));
  }

  pub fn resolved(&self) -> bool {
    self.delayed.is_none()
  }
}

impl<T> Unpin for Resolver<T> {}

impl<T> Drop for Resolver<T> {
  fn drop(&mut self) {
    self.delayed.take().map(|arc| arc.resolve(Err(Oops::Hangup(HANGUP))));
  }
}

pub struct WeakResolver<T> {
  delayed: Option<Weak<Delayed<T>>>,
}

impl<T> WeakResolver<T> {
  /// Creates a new Promise and its corresponding Resolver
  pub fn new_pair() -> (WeakResolver<T>, Promise<T>) {
    let state = CachePadded::new(AtomicBool::new(false));
    let delayed = Arc::new(Delayed { state, data: UnsafeCell::new(None), event: Event::new() });
    (WeakResolver { delayed: Some(Arc::downgrade(&delayed)) },
     Promise { delayed: Some(delayed), event: None })
  }

  /// Resolves for linked promises, notifying any that are polling
  pub fn resolve(&mut self, value: Result<T, Oops>) {
    self.delayed.take().unwrap().upgrade()
      .map(|arc| arc.resolve(value));
  }
}

impl<T> Unpin for WeakResolver<T> {}

impl<T> Drop for WeakResolver<T> {
  fn drop(&mut self) {
    self.delayed.take().and_then(|weak| weak.upgrade())
      .map(|arc| arc.resolve(Err(Oops::Hangup(HANGUP))));
  }
}

/// A (pretty horrific) wrapper that lets you avoid cloning the result
/// if you don't need to.
#[derive(Clone)]
pub struct Promised<T> {
  inner: Arc<Delayed<T>>,
}

impl<T> Eq for Promised<T> {}

impl<T> Unpin for Promised<T> {}

impl<T> PartialEq for Promised<T> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
  }
}

impl<T> Promised<T> {
  /// Retrieves a reference to the inner value
  pub fn get(&self) -> &Result<T, Oops> {
    self.inner.get().unwrap()
  }
}

impl<T: Clone> Promised<T> {
  /// Consumes, returning the value inside by cloning
  pub fn eat(self) -> Result<T, Oops> {
    self.get().clone()
  }
}

/// A value that might not be there yet
struct Delayed<T> {
  state: CachePadded<AtomicBool>,
  event: Event,
  data: UnsafeCell<Option<Result<T, Oops>>>,
}

impl<T> Unpin for Delayed<T> {}

impl<T> Delayed<T> {
  fn get(&self) -> Option<&Result<T, Oops>> {
    unsafe { &*self.data.get() }.as_ref()
  }

  fn resolve(&self, value: Result<T, Oops>) {
    unsafe { *self.data.get() = Some(value); }
    self.state.store(true, Ordering::Release);
    self.event.notify_all();
  }
}
