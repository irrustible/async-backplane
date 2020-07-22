use async_backplane::*;
use futures::prelude::*;
use std::thread;

#[test]
fn works() {
    let mut d1 = Device::new();
    let d2 = Device::new();

    d1.link(d2.open_line()).expect("failed linking");

    let i1 = d1.device_id();
    let m1 = d1.monitoring(future::pending());

    let t1 = thread::spawn(move || smol::block_on(d2.disconnect(Disconnect::Complete)));
    let t2 = thread::spawn(|| smol::block_on(m1));

    let _ = t1.join().unwrap();
    let r2 = t2.join().unwrap();

    match r2.unwrap_err() {
        Crash::Cascade(id, Disconnect::Complete) => assert_eq!(id, i1),
        _ => panic!("unexpected"),
    }
}
