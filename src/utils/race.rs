use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Like `futures_lite::future::race` but with a left bias
pub fn biased_race<T, A, B>(future1: A, future2: B) -> BiasedRace<A, B>
where
    A: Future<Output = T>,
    B: Future<Output = T>,
{
    BiasedRace { future1, future2 }
}

pin_project! {
    /// Like `futures_lite::future::Race`, but with a left bias
    #[derive(Debug)]
    pub struct BiasedRace<A, B> {
        #[pin]
        future1: A,
        #[pin]
        future2: B,
    }
}

impl<T, A, B> Future for BiasedRace<A, B>
where
    A: Future<Output = T>,
    B: Future<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Poll::Ready(t) = this.future2.poll(cx) {
            return Poll::Ready(t);
        }
        if let Poll::Ready(t) = this.future1.poll(cx) {
            return Poll::Ready(t);
        }
        Poll::Pending
    }
}
