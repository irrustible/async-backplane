use async_channel::Sender;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;

#[cfg(not(feature = "nightly"))]
use std::marker::PhantomData;

/// Future for sending lots of the same message
pub struct BulkSend<T> {
    value: T,
    inner: Vec<(bool,Sending<T>)>,
    done: bool,
}

impl<T: 'static + Send + Clone> BulkSend<T> {
    pub fn new(value: T) -> BulkSend<T> {
        BulkSend { value, inner: Vec::new(), done: false }
    }
    pub fn add_sender(&mut self, sender: Sender<T>) {
        self.inner.push((false, Sending::new(sender, self.value.clone())));
        self.done = false;
    }
}

impl<T: 'static + Send + Clone> Future for BulkSend<T> {
    type Output = ();
   fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if this.done {
            Poll::Ready(())
        } else {
            let mut dirty = false;
            for mut item in this.inner.iter_mut() {
                if !item.0 {
                    let pin = unsafe { Pin::new_unchecked(&mut item.1) };
                    match Sending::poll(pin, ctx) {
                        Poll::Pending => { dirty = true; }
                        Poll::Ready(_) => { item.0 = true; }
                    }
                }
            }
            if dirty {
                Poll::Pending
            } else {
                this.done = true;
                Poll::Ready(())
            }
        }
    }
}

#[cfg(not(feature = "nightly"))]
type Quiet = Pin<Box<dyn Future<Output = ()> + 'static + Send>>;

#[cfg(not(feature = "nightly"))]
pin_project! {
    pub struct Sending<T> {
        #[pin]
        inner: Quiet,
        _phantom: PhantomData<T>,
    }
}

#[cfg(not(feature = "nightly"))]
impl<T: 'static + Send> Sending<T> {
    #[allow(unused_must_use)]
    pub fn new(sender: Sender<T>, value: T) -> Sending<T> {
        Sending {
            inner: Box::pin(async move { sender.send(value).await; }),
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
type Quiet<T> = impl Future<Output=()>;

#[allow(unused_must_use)]
#[cfg(feature = "nightly")]
fn quiet<T: 'static + Send>(sender: Sender<T>, value: T) -> Quiet<T> {
    async move { sender.send(value).await; }
}

#[cfg(feature = "nightly")]
pin_project! {
    pub struct Sending<T> {
        #[pin]
        inner: Quiet<T>,
    }
}

#[cfg(feature = "nightly")]
impl<T: 'static + Send> Sending<T> {
    pub fn new(sender: Sender<T>, value: T) -> Sending<T> {
        Sending { inner: quiet(sender, value) }
    }
}

impl<T: 'static + Send> Future for Sending<T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        let this = self.project();
        Quiet::poll(this.inner, ctx)
    }
}
