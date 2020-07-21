use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_lite::stream::Stream;
use std::panic;
use pin_project_lite::pin_project;

use crate::{BulkSend, Crash, Device, DeviceID, Disconnect, DontPanic, Pluggable};

pin_project! {
    pub struct Managed<F: Future> {
        #[pin]
        fut: DontPanic<F>,
        device: Option<Device>,
        sending: Option<BulkSend<(DeviceID, Disconnect)>>,
    }
}

impl<F: Future> Managed<F> {
    pub fn new(fut: F, device: Device) -> Self {
        Managed {
            fut: DontPanic::new(fut),
            device: Some(device),
            sending: None,
        }
    }
}

impl<F, C, T> Future for Managed<F>
where F: Future<Output=Result<T, C>>,
      C: 'static + Send {
    type Output = ();
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        let mut this = self.project();
        if let Some(ref mut device) = &mut this.device {
            loop {
                match Device::poll_next(Pin::new(device), ctx) {
                    Poll::Ready(Some((id, disconnect))) => {
                        if disconnect.crashed() {
                            let disco = Disconnect::Cascade(device.device_id());
                            device.plugboard.broadcast(device.device_id(), disco);
                            #[allow(unused_must_use)]
                            if let Some(crashes) = &device.crashes {
                                crashes.try_send((device.device_id(), Crash::Cascade(id, disconnect)));
                            }
                            return Poll::Ready(());
                        }
                    }
                    Poll::Pending => {
                        return match DontPanic::poll(this.fut, ctx) {
                            Poll::Pending => Poll::Pending,
                            Poll::Ready(Ok(Ok(_))) => Poll::Ready(()),

                            Poll::Ready(Ok(Err(val))) => {
                                #[allow(unused_must_use)]
                                if let Some(crashes) = &device.crashes {
                                    crashes.try_send((device.device_id(), Crash::Fail(Box::new(val))));
                                }
                                Poll::Ready(())
                            }

                            Poll::Ready(Err(unwind)) => {
                                #[allow(unused_must_use)]
                                if let Some(crashes) = &device.crashes {
                                    crashes.try_send((device.device_id(), Crash::Panic(unwind)));
                                }
                                Poll::Ready(())
                            }
                        }
                    }
                    Poll::Ready(None) => unreachable!(),
                }
            }
        } else if let Some(ref mut sending) = &mut this.sending {
            let pin = unsafe { Pin::new_unchecked(sending) };
            BulkSend::poll(pin, ctx)
        } else {
            Poll::Pending // We have already completed
        }
    }
}
