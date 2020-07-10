use futures::channel::oneshot::{self, Canceled};
use futures::prelude::*;
use std::io;
use tingle::*;

#[test]
fn works() {
    let (sender, recver) = oneshot::channel();
    drop(sender);

    let mut p1 = Particle::new::<(), io::Error>(future::pending());
    let p2 = Particle::new::<(), io::Error>(future::pending());
    let mut p3 = Particle::new::<(), Canceled>(recver);
    let p4 = Particle::new::<(), io::Error>(async {
        panic!("catch that");
    });
    let p5 = Particle::new::<(), io::Error>(future::pending());
    let p6 = Particle::new::<(), io::Error>(future::ready(Ok(())));

    p1.entangle(p2.as_wave());
    p3.entangle(p2.as_wave());
    p5.as_wave().cancel();

    smol::run(async {
        // > Error::Particle(oneshot canceled)
        assert!(p3.await.is_err());
        // > Error::Entangled (p3 -> w2 && p1 <- w2)
        assert!(p1.await.is_err());
        // > Error::Entangled (p3 -> w2)
        assert!(p2.await.is_err());
        // > Error::Panic(Any)
        assert!(p4.await.is_err());
        // > Ok(None)
        assert_eq!(p5.await.unwrap(), None);
        // > Ok(Some(()))
        assert_eq!(p6.await.unwrap(), Some(()));
    });
}
