use crate::Message;
use core::fmt::Debug;

/// A result from `watch()`.
#[derive(Debug)]
pub enum Watched<T: Debug> {
    /// The provided Future completed.
    Completed(T),
    /// A message was received.
    Messaged(Message),
}

use Watched::{Completed, Messaged};

impl<T: Debug> Watched<T> {
    /// True if the future completed.
    pub fn is_completed(&self) -> bool { matches!(self, Completed(_)) }

    /// True if we received a message.
    pub fn is_messaged(&self) -> bool { matches!(self, Messaged(_)) }

    /// Take the completed result or panic.
    pub fn unwrap_completed(self) -> T {
        if let Completed(c) = self {
            c
        } else {
            panic!("Watched is not Completed");
        }
    }

    /// Take the received message or panic.
    pub fn unwrap_messaged(self) -> Message {
        if let Messaged(m) = self {
            m
        } else {
            panic!("Watched is not Messaged");
        }
    }
}

impl<T: Debug + PartialEq> PartialEq for Watched<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Completed(l), Completed(r)) => *l == *r,
            (Messaged(l), Messaged(r)) => *l == *r,
            _ => false,
        }
    }
}

impl<T: Debug + Eq> Eq for Watched<T> {}
