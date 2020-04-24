use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use core::time::Duration;
use futures::future::BoxFuture;
use async_std::task::sleep;

pub struct Timer {
  inner: BoxFuture<'static, ()>,
}

impl Timer {
  pub fn after(timeout: Duration) -> Timer {
    Timer { inner: Box::pin(sleep(timeout)) }
  }
}

impl Future for Timer {
  type Output = ();

  fn poll(mut self: Pin<&mut Timer>, context: &mut Context) -> Poll<()> {
    BoxFuture::poll(Pin::new(&mut self.as_mut().inner), context)
  }

}
