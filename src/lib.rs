// use event_listener::{Event, EventListener};
// use intmap::IntMap;
use pin_project_lite::pin_project;
// #[cfg(feature = "smol")]
// use smol::Task;
use std::any::Any;
// use std::convert::TryInto;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::{Arc, Mutex};
use async_channel::{Sender, Receiver};
use futures_core::stream::Stream;

/// A Particle ID
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pid {
    inner: u64,
}

/// Status of a process's termination
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Measurement {
    /// The task completed successfully
    Success,
    /// There was an error running the task
    Error,
    /// A partner task errored
    Cascade(Pid),
}

/// The reason a process failed
#[derive(Debug)]
pub enum Error {
    Panic(Box<dyn Any + Send + 'static>),
    Failure(Box<dyn Any + Send + 'static>),
    Cascade(Pid),
}

pub type Exit = (Pid, Measurement);

pub type Crash = (Pid, Error);

// struct Supervisor {
//     tangle: Arc<Tangle>,
// }

// A particle may listen for the status of another particle

/// A Particle is an error domain for a computation. Through
/// entanglement, it can become aware of the completion (and status)
/// of computations connected to other Particles and react accordingly.
    
pub struct Particle {
    exits: Receiver<Exit>,
    inner: Arc<Particulate>,
}

impl Particle {
    pub fn new() -> Self {
        let (send, exits) = async_channel::unbounded();
        let inner = Arc::new(Particulate::new(send));
        Particle { exits, inner }
    }

    pub fn pid(&self) -> Pid {
        Pid { inner: &*self.inner as *const _ as u64 }
    }

    pub fn exit(self, measurement: Measurement) {
        self.inner.notify((self.pid(), measurement)).unwrap();
    }

    pub fn monitor(&self, who: Boson) -> bool {
        who.inner.add_monitor(Boson::new(self))
    }

    pub fn demonitor(&self, who: Boson) -> bool {
        who.inner.remove_monitor(self.pid())
    }

    pub fn add_monitor(&self, who: Boson) -> bool {
        self.inner.add_monitor(who)
    }

    pub fn remove_monitor(&self, who: Boson) -> bool {
        self.inner.remove_monitor(who.pid())
    }

    pub fn entangle(&self, with: Boson) {
        self.monitor(with.clone());
        self.add_monitor(with);
    }
}

impl Unpin for Particle {}

impl Stream for Particle {
    type Item = Exit;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Exit>> {
        Receiver::poll_next(Pin::new(&mut Pin::into_inner(self).exits), ctx)
    }
}

#[derive(Clone)]
pub struct Boson {
    inner: Arc<Particulate>,
}

impl Boson {
    pub fn new(particle: &Particle) -> Self {
        Boson { inner: particle.inner.clone() }
    }

    pub fn pid(&self) -> Pid {
        Pid { inner: &*self.inner as *const _ as u64 }
    }

    pub fn monitor(&self, to: Boson) -> bool {
        to.inner.add_monitor(self.clone())
    }

    pub fn add_monitor(&self, who: Boson) -> bool {
        self.inner.add_monitor(who)
    }

    pub fn entangle(&self, with: Boson) {
        self.monitor(with.clone());
        self.add_monitor(with);
    }

    pub fn notify(&self, exit: Exit) -> Result<(), ()> {
        self.inner.notify(exit)
    }
}

impl PartialEq for Boson {
    fn eq(&self, other: &Boson) -> bool {
        self.pid() == other.pid()
    }
}

impl Eq for Boson {}

struct Particulate {
    me: Sender<Exit>,
    bosons: Mutex<Bosons>,
}

impl Particulate {
    fn new(me: Sender<Exit>) -> Self {
        let bosons = Mutex::new(Bosons::new());
        Particulate { me, bosons }
    }
    fn add_monitor(&self, boson: Boson) -> bool {
        self.bosons.lock().unwrap().add(boson)
    }
    fn remove_monitor(&self, pid: Pid) -> bool {
        self.bosons.lock().unwrap().remove(pid)
    }
    fn broadcast(&self, exit: Exit) {
        self.bosons.lock().unwrap().notify(exit)
    }
    fn notify(&self, exit: Exit) -> Result<(), ()> {
        self.me.try_send(exit).map_err(|_| ())
    }
}

struct Bosons {
    inner: Vec<Option<Boson>>,
}

impl Bosons {
    fn new() -> Self {
        Bosons { inner: Vec::new() }
    }
    fn add(&mut self, boson: Boson) -> bool {
        for option in &self.inner {
            if let Some(boson2) = option {
                if boson2 == &boson {
                    return false;
                }
            }
        }
        self.inner.push(Some(boson));
        true
    }
    fn remove(&mut self, pid: Pid) -> bool {
        for b in &mut self.inner {
            if let Some(boson) = b {
                if boson.pid() == pid {
                    b.take();
                    return true;
                }
            }                
        }
        false
    }
    fn notify(&mut self, exit: Exit) {
        for b in &mut self.inner {
            if let Some(boson) = b {
                boson.notify(exit.clone()).unwrap();
            }
        }
    }
}

pin_project! {
   pub struct Supervised<F> {
       #[pin]
       fut: F,
       particle: Option<Particle>,
       error: Sender<Error>,
   }
}

// impl<F, T> Supervised<F>
// where F: Future<Output=Result<(), T>>,
//       T: Into<Error> {
//     fn report_error(&mut self, error: Error) -> Result<(), ()> {
//         self.error.try_send(error).map_err(|_| ())
//     }
//     fn report_exit(particle: Particle, measurement: Measurement) {
//         particle.exit(measurement)
//     }
//     // fn check_exit(particle: &mut Particle, pid: Pid, measurement: Measurement) -> Poll<()> {
//     //     if measurement != Measurement::Success {
//     //         particle.exit(Measurement::Cascade(pid));
//     //         Poll::Ready(())
//     //     } else {
//     //         Poll::Pending
//     //     }
//     // }
//     // fn poll_future(fut: Pin<&mut F>
// }

// impl<F, T> Future for Supervised<F>
// where F: Future<Output=Result<(), T>>,
//       T: Into<Error> {
//     type Output=();
//     fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
//         let this = self.project();
//         match this.particle.poll_next(this.particle, ctx) {
//             Poll::Ready(None) => Poll::Ready(()),
//             Poll::Ready(Some((pid, measurement))) =>
//                 Supervised::on_particle_some(&mut this.particle, pid, measurement),

//             Poll::Pending => {
//                 match catch_unwind(AssertUnwindSafe(|| this.fut.poll(ctx))) {
                    
//         //     Ok(Poll::Pending) => Poll::Pending,
//         //     Ok(Poll::Ready((ok))) => {
//         //         self.particle.broadcast(Measurement::Success)
//         //         Poll::Ready(())
//         //     }
//         //     Ok(Poll::Ready(Err(err))) => {
//         //         self.failed();
//         //         Poll::Ready(Err(Error::Error(err.into())))
//         //     }
//         //     Err(err) => {
//         //         self.particle.broadcast(Measurement::Error);
//         //         Poll::Ready(Err(Error::Panic(err)))
//         //     }
//         // }
//     }
// }

// #[cfg(feature = "smol")]
// impl<F> Particle<F> {
//     pub fn spawn<E>(fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let particle = Particle::new(fut);
//         let wave = particle.as_wave();
//         Task::spawn(ParticleTask { particle }).detach();

//         wave
//     }

            //     pub fn spawn_blocking<E>(fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let particle = Particle::new(fut);
//         let wave = particle.as_wave();
//         Task::blocking(ParticleTask { particle }).detach();

//         wave
//     }

//     pub fn spawn_local<E>(fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let particle = Particle::new(fut);
//         let wave = particle.as_wave();
//         Task::local(ParticleTask { particle }).detach();

//         wave
//     }
// }

// impl Wave {
//     pub fn cancel(&mut self) {
//         self.listener.take();
//         self.inner.succeeded();
//     }
// }

// #[cfg(feature = "smol")]
// impl Wave {
//     pub fn spawn<F, E>(&self, fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let mut particle = Particle::new(fut);
//         let wave = particle.as_wave();

//         particle.entangle(self.clone());
//         Task::spawn(ParticleTask { particle }).detach();

//         wave
//     }

//     pub fn spawn_blocking<F, E>(&self, fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + Send + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let mut particle = Particle::new(fut);
//         let wave = particle.as_wave();

//         particle.entangle(self.clone());
//         Task::blocking(ParticleTask { particle }).detach();

//         wave
//     }

//     pub fn spawn_local<F, E>(&self, fut: F) -> Wave
//     where
//         F: Future<Output = Result<(), E>> + 'static,
//         E: Into<anyhow::Error>,
//     {
//         let mut particle = Particle::new(fut);
//         let wave = particle.as_wave();

//         particle.entangle(self.clone());
//         Task::local(ParticleTask { particle }).detach();

//         wave
//     }
// }

// impl<F> Particle<F> {
//     fn succeeded(self: Pin<&mut Self>) {
//         self.inner.succeeded();
//         self.entangled_succeeded(false);
//     }

//     fn failed(self: Pin<&mut Self>) {
//         self.inner.failed();
//         self.entangled_failed(false);
//     }

//     // Returns `true` if the new status of the particle and its entangled waves is equal to
//     // `ENTANGLED_SUCCEEDED` or `SUCCEEDED`, or `false` if it is equal to `ENTANGLED_FAILED` or
//     // `FAILED`.
//     fn entangled_succeeded(mut self: Pin<&mut Self>, inner: bool) -> bool {
//         if inner && !self.inner.entangled_succeeded() {
//             self.entangled_failed(false);
//             return false;
//         }

//         let mut failed = false;
//         for wave in self.as_mut().project().entangled.values_mut() {
//             if !wave.inner.entangled_succeeded() {
//                 failed = true;
//                 break;
//             }
//         }

//         if failed {
//             self.entangled_failed(false);
//             false
//         } else {
//             self.project().entangled.clear();
//             true
//         }
//     }

//     fn entangled_failed(self: Pin<&mut Self>, inner: bool) {
//         if inner {
//             self.inner.entangled_failed();
//         }

//         for (_, wave) in self.project().entangled.drain() {
//             wave.inner.entangled_failed();
//         }
//     }
// }

// impl Inner {
//     fn succeeded(&self) {
//         let mut status = RUNNING;
//         loop {
//             // [...] -> SUCCEEDED -> FAILED
//             match self.status.compare_exchange_weak(
//                 status,
//                 SUCCEEDED,
//                 Ordering::SeqCst,
//                 Ordering::SeqCst,
//             ) {
//                 Ok(RUNNING) => {
//                     self.event.notify(!0);
//                     break;
//                 }
//                 Ok(_) | Err(FAILED) | Err(SUCCEEDED) => break,
//                 Err(cur) => status = cur,
//             }
//         }
//     }

//     fn failed(&self) {
//         // [...] -> FAILED
//         if self.status.swap(FAILED, Ordering::SeqCst) == RUNNING {
//             self.event.notify(!0);
//         }
//     }

//     // Returns `true` if the new status is equal to `ENTANGLED_SUCCEEDED` or `SUCCEEDED`, or `false`
//     // if it is equal to `ENTANGLED_FAILED` or `FAILED`.
//     fn entangled_succeeded(&self) -> bool {
//         // RUNNING -> ENTANGLED_SUCCEEDED -> [...]
//         match self
//             .status
//             .compare_and_swap(RUNNING, ENTANGLED_SUCCEEDED, Ordering::SeqCst)
//         {
//             RUNNING => {
//                 self.event.notify(!0);
//                 true
//             }
//             FAILED | ENTANGLED_FAILED => false,
//             _ => true,
//         }
//     }

//     fn entangled_failed(&self) {
//         let mut status = RUNNING;
//         loop {
//             // [...] -> ENTANGLED_FAILED -> SUCCEEDED -> FAILED
//             match self.status.compare_exchange_weak(
//                 status,
//                 ENTANGLED_FAILED,
//                 Ordering::SeqCst,
//                 Ordering::SeqCst,
//             ) {
//                 Ok(RUNNING) => {
//                     self.event.notify(!0);
//                     break;
//                 }
//                 Ok(_) | Err(ENTANGLED_FAILED) | Err(SUCCEEDED) | Err(FAILED) => break,
//                 Err(cur) => status = cur,
//             }
//         }
//     }
// }

// impl<F, T, E> Future for Particle<F>
// where
//     F: Future<Output = Result<T, E>>,
//     E: Into<anyhow::Error>,
// {
//     type Output = Result<Option<T>, Error>;

//     fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
//         let this = self.as_mut().project();

//         // Check the particle's current state and eventually propagate it to its entangled waves.
//         match this.inner.status.load(Ordering::SeqCst) {
//             SUCCEEDED | ENTANGLED_SUCCEEDED => return Poll::Ready(Ok(None)),
//             FAILED => {
//                 self.entangled_failed(false);
//                 return Poll::Ready(Err(Error::ParticleFailure));
//             }
//             ENTANGLED_FAILED => {
//                 self.entangled_failed(false);
//                 return Poll::Ready(Err(Error::EntangledFailure));
//             }
//             _ => (),
//         }

//         // Check for entangled waves whose status is `SUCCEEDED` or `ENTANGLED_SUCCEEDED` and stop
//         // when finding one whose status is `FAILED` or `ENTANGLED_FAILED`.
//         let mut status = RUNNING;
//         for mut wave in this.entangled.values_mut() {
//             match Pin::new(&mut wave).poll(ctx) {
//                 Poll::Ready(Ok(())) => status = SUCCEEDED,
//                 Poll::Ready(Err(_)) => {
//                     status = FAILED;
//                     break;
//                 }
//                 Poll::Pending => (),
//             }
//         }

//         // If at least one entangled wave's status wasn't `RUNNING`, update the status of the
//         // particle and all its entangled waves.
//         match status {
//             SUCCEEDED => {
//                 if self.entangled_succeeded(true) {
//                     return Poll::Ready(Ok(None));
//                 } else {
//                     return Poll::Ready(Err(Error::EntangledFailure));
//                 }
//             }
//             FAILED => {
//                 self.entangled_failed(true);
//                 return Poll::Ready(Err(Error::EntangledFailure));
//             }
//             _ => (),
//         }

//         // Otherwise, poll the inner future and eventually update statuses.
//         match catch_unwind(AssertUnwindSafe(|| this.fut.poll(ctx))) {
//             Ok(Poll::Pending) => Poll::Pending,
//             Ok(Poll::Ready(Ok(ok))) => {
//                 self.succeeded();
//                 Poll::Ready(Ok(Some(ok)))
//             }
//             Ok(Poll::Ready(Err(err))) => {
//                 self.failed();
//                 Poll::Ready(Err(Error::Error(err.into())))
//             }
//             Err(err) => {
//                 self.failed();
//                 Poll::Ready(Err(Error::Panic(err)))
//             }
//         }
//     }
// }

// impl<F, T, E> Future for ParticleTask<F>
// where
//     F: Future<Output = Result<T, E>>,
//     E: Into<anyhow::Error>,
// {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
//         match self.project().particle.poll(ctx) {
//             Poll::Ready(_) => Poll::Ready(()),
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }

// impl Future for Wave {
//     type Output = Result<(), Error>;

//     fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
//         let mut listener = if let Some(listener) = self.listener.take() {
//             listener
//         } else {
//             // Create a new `EventListener` first to be sure that the wave's status isn't updated
//             // between the time we first check it and the time we finish creating the listener.
//             let listener = self.inner.event.listen();
//             match self.inner.status.load(Ordering::SeqCst) {
//                 SUCCEEDED | ENTANGLED_SUCCEEDED => return Poll::Ready(Ok(())),
//                 FAILED => return Poll::Ready(Err(Error::ParticleFailure)),
//                 ENTANGLED_FAILED => return Poll::Ready(Err(Error::EntangledFailure)),
//                 _ => listener,
//             }
//         };

//         if Pin::new(&mut listener).poll(ctx).is_ready() {
//             match self.inner.status.load(Ordering::SeqCst) {
//                 SUCCEEDED | ENTANGLED_SUCCEEDED => Poll::Ready(Ok(())),
//                 FAILED => Poll::Ready(Err(Error::ParticleFailure)),
//                 ENTANGLED_FAILED => Poll::Ready(Err(Error::EntangledFailure)),
//                 _ => unreachable!(),
//             }
//         } else {
//             self.listener = Some(listener);
//             Poll::Pending
//         }
//     }
// }

// impl Clone for Wave {
//     fn clone(&self) -> Self {
//         Wave {
//             inner: self.inner.clone(),
//             listener: None,
//         }
//     }
// }
