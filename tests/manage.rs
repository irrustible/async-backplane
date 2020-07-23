use async_backplane::*;
use futures_lite::*;
use futures_lite::future::block_on;
use std::{io, thread};

#[test]
fn test() {
    let d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();

    d1.link(d2.open_line()).expect("failed linking");

    let t1 = thread::spawn(move || block_on(d2.disconnect(Disconnect::Crash)));
    let t2: thread::JoinHandle<Result<(), Crash<io::Error>>> =
        thread::spawn(move || block_on(d1.manage(future::pending::<io::Result<()>>())));

    assert_eq!((), t1.join().unwrap());

    match t2.join().unwrap().unwrap_err() {
        Crash::Cascade(id, Disconnect::Crash) => assert_eq!(id, i2),
        other => {
            println!("{:?}", &other);
            panic!("unexpected");
        }
    }
}
