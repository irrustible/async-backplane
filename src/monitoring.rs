use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_lite::stream::Stream;
use std::panic;
use pin_project_lite::pin_project;

use crate::{Crash, Device, DontPanic};

pin_project! {
    /// Wraps a Future such that it will return `Err(Crash<C>)` if it
    /// crashes or one of the Futures it is monitoring crashes.
    pub struct Monitoring<'a, F: Future> {
        #[pin]
        fut: DontPanic<F>,
        device: Option<&'a mut Device>
    }
}

impl<'a, F: Future> Monitoring<'a, F> {
    pub fn new(fut: F, device: &'a mut Device) -> Self {
        Monitoring { fut: DontPanic::new(fut), device: Some(device) }
    }
}

impl<'a, F, C, T> Future for Monitoring<'a, F>
where F: Future<Output=Result<T, C>>,
      C: 'static + Send {
    type Output = Result<T, Crash<C>>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(ref mut device) = &mut this.device {
            loop {
                match Device::poll_next(Pin::new(device), ctx) {
                    Poll::Ready(Some((id, disconnect))) => {
                        if disconnect.crashed() {
                            return Poll::Ready(Err(Crash::Cascade(id, disconnect)));
                        }
                    }
                    Poll::Pending => {
                        return match DontPanic::poll(this.fut, ctx) {

                            Poll::Pending => Poll::Pending,

                            Poll::Ready(Ok(Ok(val))) => Poll::Ready(Ok(val)),

                            Poll::Ready(Ok(Err(val))) =>
                                Poll::Ready(Err(Crash::Fail(val))),

                            Poll::Ready(Err(unwind)) =>
                                Poll::Ready(Err(Crash::Panic(unwind))),

                        }
                    }
                    Poll::Ready(None) => unreachable!(),
                }
            }
        } else {
            Poll::Pending // We have already completed
        }
    }
}
