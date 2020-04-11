use crate::Pid;
use std::fmt::Debug;

/// Created when a process exits abnormally.
#[derive(Clone)]
pub struct Bad<Err: Clone + Debug> {

  /// The associated error
  pub error: Err,

  /// The pid of the exited process
  pub pid: Pid,

  /// The process that caused the process to exit, either the pid of
  /// the process which exited or the pid of the process which
  /// requested the process exit
  pub source: Pid,

}
