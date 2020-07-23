// use async_backplane::*;
// use futures_lite::*;
// use futures_lite::future::block_on;
// use std::{io, thread};

// #[cfg(feature = "smol")]
// #[test]
// fn smol() {
//     let (sender, recver) = oneshot::channel();
//     drop(sender);

//     let w2 = Particle::spawn::<io::Error>(future::pending());
//     let w1 = w2.spawn::<_, io::Error>(future::pending());
//     let w3 = w2.spawn::<_, Canceled>(recver);
//     let w4 = Particle::spawn::<io::Error>(async {
//         panic!("catch that");
//     });
//     let mut w5 = Particle::spawn::<io::Error>(future::pending());
//     let w6 = Particle::spawn::<io::Error>(future::ready(Ok(())));

//     w5.cancel();

//     smol::run(async {
//         // > Error::Particle(oneshot canceled)
//         assert!(w3.await.is_err());
//         // > Error::Entangled (p3 -> w2 && p1 <- w2)
//         assert!(w1.await.is_err());
//         // > Error::Entangled (p3 -> w2)
//         assert!(w2.await.is_err());
//         // > Error::Panic(Any)
//         assert!(w4.await.is_err());
//         // > Ok(())
//         assert!(w5.await.is_ok());
//         // > Ok(())
//         assert!(w6.await.is_ok());
//     });
// }
