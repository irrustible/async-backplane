use std::fmt::{self, Display, Formatter};
use std::result::Result;
use global_counter::primitive::exact::CounterUsize;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Debug)]
pub struct Pid {
  inner: usize,
}

impl Display for Pid {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    f.write_fmt(format_args!("<@{}>", self.inner))
  }
}

impl Pid {
  pub fn next() -> Pid {
    static COUNTER: CounterUsize = CounterUsize::new(0);
    Pid { inner: COUNTER.inc() }
  }
}
