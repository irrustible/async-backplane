use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use core::time::Duration;
use tokio::time::{delay_for, Delay};

pub struct Timer {
  inner: Delay,
}

impl Timer {
  pub fn after(timeout: Duration) -> Timer {
    Timer { inner: delay_for(timeout) }
  }
}

impl Future for Timer {
  type Output = ();

  fn poll(mut self: Pin<&mut Timer>, context: &mut Context) -> Poll<()> {
    Delay::poll(Pin::new(&mut self.as_mut().inner), context)
  }

}
