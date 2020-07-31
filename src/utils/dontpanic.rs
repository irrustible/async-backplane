use maybe_unwind::{maybe_unwind, Unwind};
use pin_project_lite::pin_project;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Wraps a Future such that it traps panics
pub fn dont_panic<F: Future>(f: F) -> DontPanic<F> {
    DontPanic::new(f)
}

pin_project! {
    /// Future for `dont_panic()`
    pub struct DontPanic<F: Future> {
        #[pin]
        fut: F,
    }
}

impl<F: Future> DontPanic<F> {
    /// Creates a new DontPanic wrapping the provided Future
    pub fn new(fut: F) -> Self {
        DontPanic { fut }
    }
}

impl<F, T> Future for DontPanic<F>
where
    F: Future<Output = T>,
{
    type Output = Result<T, Unwind>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        match maybe_unwind(AssertUnwindSafe(|| this.fut.poll(ctx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Err(unwind) => Poll::Ready(Err(unwind)),
        }
    }
}
