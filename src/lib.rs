// #![feature(generators, generator_trait)]
// #![feature(async_closure)]
// #![feature(drain_filter)]
// #![feature(proc_macro_hygiene)]

// use std::ops::{Generator, GeneratorState};
// use std::pin::Pin;

// mod join;
// pub use join::Join;

// pub mod process;

// pub mod rental_future;
// pub mod channel;
// pub mod supervisor;
// use futures::Future;

// use std::sync::atomic::{AtomicU64, Ordering};
use std::error;
use std::fmt::Debug;

// use std::time::Duration;

// use thiserror::Error;


mod pid;
pub use pid::Pid;

mod links;
pub use links::{Link, Linking, Links};

mod bad;
pub use bad::Bad;

mod exit;
pub use exit::Exit;

mod control_handle;

pub struct Quantum<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  pub pid: Pid,
  // pending: PendingLinks
  links: Links<Ret, Err>, // quanta with which we are entangled
}

/*

 A process is a single logical thread of execution.

 It exchanges control messages with other processes in order that they
 can be informed of exits.

A process needs to keep a list of processes linked to it so that when
it exits, it can notify all the linked processes. It also needs to
keep a list of processes that it is linked to and the recovery
behaviour for each.

A link is meant to be something you can rely on, but is is
asynchronous. To apply synchronous semantics to it, you can specify a
timeout.

In order to link or cancel a process, you need a control handle

The purpose of erlang is to provide a reliable control backplane.

A process may entangle itself with another process so their fates are linked.

*/


// /// Remove me when it lands for real
// pub trait IntoFuture {
//   type Output;
//   type Future: Future<Output = Self::Output>;
//   fn into_future(self) -> Self::Future;
// }

