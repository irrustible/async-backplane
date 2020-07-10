use futures::channel::oneshot::{self, Canceled};
use futures::prelude::*;
use std::io;
use tingle::*;

#[test]
fn works() {
    let (sender, recver) = oneshot::channel();

    let mut p1 = Particle::new::<(), io::Error>(future::pending());
    let p2 = Particle::new::<(), Canceled>(future::pending());
    let mut p3 = Particle::new::<(), Canceled>(recver);
    let p4 = Particle::new::<(), io::Error>(async {
        panic!("catch that");
    });

    p1.entangle(p2.as_wave());
    p3.entangle(p2.as_wave());

    drop(sender);

    // > Error::Particle(oneshot canceled)
    assert!(smol::block_on(p3).is_err());
    // > Error::Entangled (p3 -> w2 && p1 <- w2)
    assert!(smol::block_on(p1).is_err());
    // > Error::Entangled (p3 -> w2)
    assert!(smol::block_on(p2).is_err());
    // > Error::Panic(Any)
    println!("{:?}", smol::block_on(p4));
}
