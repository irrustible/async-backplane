///! Copied and tweaked from futures_util to avoid dependency on futures
///! Licensed: MIT / Apache2 dual
use core::any::Any;
use core::pin::Pin;
use std::panic::{catch_unwind, UnwindSafe, AssertUnwindSafe};

use std::future::Future;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    #[derive(Debug)]p
    pub struct CatchUnwind<Fut> {
        #[pin]
        inner: Fut,
    }
}

impl<Fut> CatchUnwind<Fut> where Fut: Future + UnwindSafe {
    pub(super) fn new(future: Fut) -> CatchUnwind<Fut> {
        CatchUnwind { inner: future }
    }
}

impl<Fut> Future for CatchUnwind<Fut>
    where Fut: Future + UnwindSafe,
{
    type Output = Result<Fut::Output, Box<dyn Any + Send>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let f = self.project().inner;
        catch_unwind(AssertUnwindSafe(|| f.poll(cx)))?.map(Ok)
    }
}
