use async_backplane::*;
use futures::prelude::*;
use std::{io, thread};

#[test]
fn works() {
    let mut d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();

    d1.link(d2.open_line()).expect("failed linking");

    let t1 = thread::spawn(move || smol::block_on(d2.disconnect(Disconnect::Crash)));
    let t2 =
        thread::spawn(move || smol::block_on(d1.monitoring(future::pending::<io::Result<()>>())));

    let _ = t1.join().unwrap();
    let r2 = t2.join().unwrap();

    match r2.unwrap_err() {
        Crash::Cascade(id, Disconnect::Crash) => assert_eq!(id, i2),
        _ => panic!("unexpected"),
    }
}
