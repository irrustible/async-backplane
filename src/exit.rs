use crate::{Bad, Pid};
use std::fmt::Debug;

#[derive(Clone)]
pub enum Exit<Ret, Err>
where Ret: Clone, Err: Clone + Debug {
  Good(Pid, Ret),
  Bad(Pid, Bad<Err>),
}
