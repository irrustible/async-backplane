use core::fmt::{self, Display, Formatter};
use core::result::Result;
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
/// A process identifier
pub struct Name {
  pub(crate) inner: u64,
}

impl Unpin for Name {}

impl Name {
  /// Increment the global counter atomically, returning the old value
  pub fn next() -> Name {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    Name { inner: COUNTER.fetch_add(1, Ordering::SeqCst) }
  }

  pub(crate) fn new(inner: u64) -> Name {
    Name { inner }
  }
}

impl Display for Name {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    f.write_fmt(format_args!("Name({})", self.inner))
  }
}

