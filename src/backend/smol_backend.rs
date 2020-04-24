use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use core::time::Duration;
use smol;

pub struct Timer {
  inner: smol::Timer,
}

impl Timer {
  pub fn after(timeout: Duration) -> Timer {
    Timer { inner: smol::Timer::after(timeout) }
  }
}

impl Future for Timer {
  type Output = ();

  fn poll(mut self: Pin<&mut Timer>, context: &mut Context) -> Poll<()> {
    match smol::Timer::poll(Pin::new(&mut self.as_mut().inner), context) {
      Poll::Ready(_) => Poll::Ready(()),
      Poll::Pending => Poll::Pending,
    }
  }

}
