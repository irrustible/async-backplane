use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::panic::AssertUnwindSafe;
use maybe_unwind::{maybe_unwind, Unwind};
pin_project! {
    /// Wraps a Future such that it traps panics
    pub struct DontPanic<F: Future> {
        #[pin]
        fut: F,
    }
}

impl<F: Future> DontPanic<F> {
    pub fn new(fut: F) -> Self {
        DontPanic { fut }
    }
}

impl<F, T> Future for DontPanic<F>
where F: Future<Output=T> {
    type Output = Result<T, Unwind>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        match maybe_unwind(AssertUnwindSafe(|| this.fut.poll(ctx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Err(unwind) => Poll::Ready(Err(unwind))
        }
    }
}
