use core::cell::UnsafeCell;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};
use event_listener::{Event, EventListener};
use futures::prelude::*;
use intmap::IntMap;
use piper::{Receiver, Sender};
use std::sync::Arc;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Name(u64);

pub trait Superposition: Clone + Unpin + Send + Sync {}
impl<T: Clone + Unpin + Send + Sync> Superposition for T {}

const DEFAULT_CHAN_CAP: usize = 8;

pub struct Quantum<S: Superposition> {
    name: Name,
    sender: Sender<Boson<S>>,
    recver: Receiver<Boson<S>>,
    delay: Event,
    delayed: Arc<Delayed<S>>,
    observables: IntMap<Observable<S>>,
    observed: Vec<S>,
}

struct Delayed<S: Superposition> {
    val: UnsafeCell<Option<S>>,
}

#[derive(Debug)]
struct Observable<S: Superposition> {
    tangle: Tangle<S>,
    entanglement: Entanglement,
}

pub struct Tangle<S: Superposition> {
    name: Name,
    sender: Sender<Boson<S>>,
    delay: Option<EventListener>,
    delayed: Arc<Delayed<S>>,
}

enum Boson<S: Superposition> {
    Entangle(Tangle<S>),
    Untangle(Name),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Entanglement {
    Observer,
    Entangled,
}

impl Name {
    pub fn random() -> Self {
        Name(fastrand::u64(..))
    }
}

impl<S: Superposition> Quantum<S> {
    pub fn new(name: Name) -> Self {
        Self::with_chan_cap(name, DEFAULT_CHAN_CAP)
    }

    pub fn with_chan_cap(name: Name, cap: usize) -> Self {
        let (sender, recver) = piper::chan(cap);

        Quantum {
            name,
            sender,
            recver,
            delay: Event::new(),
            delayed: Arc::new(Delayed::new()),
            observables: IntMap::new(),
            observed: Vec::new(),
        }
    }

    pub fn name(&self) -> Name {
        self.name
    }

    pub fn tangle(&self) -> Tangle<S> {
        Tangle {
            name: self.name,
            sender: self.sender.clone(),
            delay: Some(self.delay.listen()),
            delayed: self.delayed.clone(),
        }
    }

    pub async fn entangle(&mut self, with: Tangle<S>, entanglement: Entanglement) {
        if self.observables.contains_key(*with.name) {
            return;
        }

        if entanglement.is_entangled() {
            with.sender.send(Boson::Entangle(self.tangle())).await;
        }

        self.observables.insert(
            *with.name,
            Observable {
                tangle: with,
                entanglement,
            },
        );
    }

    pub async fn untangle(&mut self, from: Name) {
        match self.observables.remove(*from) {
            Some(observable) if observable.entanglement.is_entangled() => {
                observable
                    .tangle
                    .sender
                    .send(Boson::Untangle(self.name))
                    .await
            }
            _ => (),
        }
    }

    pub fn exit(self, value: S) {
        // This is safe because only this method writes to `val`, it is guaranteed
        // that other methods won't read from it before `notify()` is called, and
        // it is guaranteed that this method can't be called twice (since it takes
        // `self`).
        let val = unsafe { self.delayed.val.get().as_mut().unwrap() };
        *val = Some(value);

        self.delay.notify(!0);
    }
}

impl<S: Superposition> Delayed<S> {
    fn new() -> Self {
        Delayed {
            val: UnsafeCell::new(None),
        }
    }
}

impl<S: Superposition> Tangle<S> {
    pub fn name(&self) -> Name {
        self.name
    }
}

impl Entanglement {
    pub fn is_observer(self) -> bool {
        self == Entanglement::Observer
    }

    pub fn is_entangled(self) -> bool {
        self == Entanglement::Entangled
    }
}

impl<S: Superposition> Stream for Quantum<S> {
    type Item = S;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        while let Some(boson) = self.recver.try_recv() {
            match boson {
                Boson::Entangle(with) => {
                    self.observables
                        .insert(
                            *with.name,
                            Observable {
                                tangle: with,
                                entanglement: Entanglement::Entangled,
                            },
                        );
                },
                Boson::Untangle(from) => {
                    self.observables
                        .remove(*from);
                },
            }
        }

        if let observed @ Some(_) = self.observed.pop() {
            return Poll::Ready(observed);
        }

        let mut exited = Vec::new();
        for (name, observable) in self.observables.iter_mut() {
            if let Poll::Ready(observed) = Pin::new(&mut observable.tangle).poll(ctx) {
                exited.push((*name, observed));
            }
        }

        if exited.len() > 1 {
            for (name, observed) in exited.drain(1..) {
                self.observables.remove(name);
                self.observed.push(observed);
            }
        }

        if let Some((_, observed)) = exited.pop() {
            Poll::Ready(Some(observed))
        } else {
            Poll::Pending
        }
    }
}

impl<S: Superposition> Future for Tangle<S> {
    type Output = S;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        if let Some(delay) = self.delay.as_mut() {
            match Pin::new(delay).poll(ctx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(()) => {
                    self.delay.take();

                    // This is safe because, if a notification has been received, then
                    // we know that `exit()` has been called and this method guarantees
                    // that `val` won't written to after that.
                    let val = unsafe { self.delayed.val.get().as_ref().unwrap() };
                    Poll::Ready(val.clone().unwrap())
                }
            }
        } else {
            // This is safe because, if a notification has been received, then
            // we know that `exit()` has been called and this method guarantees
            // that `val` won't written to after that.
            let val = unsafe { self.delayed.val.get().as_ref().unwrap() };
            Poll::Ready(val.clone().unwrap())
        }
    }
}

impl From<u64> for Name {
    fn from(name: u64) -> Self {
        Name(name)
    }
}

impl Deref for Name {
    type Target = u64;

    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl<S: Superposition + Debug> Debug for Quantum<S> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Quantum")
            .field("name", &self.name)
            .field("sender", &self.sender)
            .field("recver", &self.recver)
            .field("delay", &self.delay)
            .field("delayed", &Option::<S>::None)
            .field("observables", &self.observables)
            .field("observed", &self.observed)
            .finish()
    }
}

impl<S: Superposition + Debug> Debug for Tangle<S> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Tangle");
        fmt.field("name", &self.name)
            .field("sender", &self.sender)
            .field("delay", &self.delay);

        if self.delay.is_some() {
            fmt.field("delayed", &Option::<S>::None);
        } else {
            // This is safe because, if a notification has been received, then
            // we know that `exit()` has been called and this method guarantees
            // that `val` won't written to after that.
            fmt.field("delayed", unsafe { &*self.delayed.val.get() });
        }

        fmt.finish()
    }
}

unsafe impl<S: Superposition> Send for Delayed<S> {}
unsafe impl<S: Superposition> Sync for Delayed<S> {}
