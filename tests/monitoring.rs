use async_backplane::*;
use futures_lite::*;
use futures_lite::future::block_on;
use std::{io, thread};

#[test]
fn test() {
    let mut d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();

    d1.link(d2.open_line()).expect("failed linking");

    let t1 = thread::spawn(move || block_on(d2.disconnect(Disconnect::Crash)));
    let t2: thread::JoinHandle<Result<io::Result<()>, Result<(DeviceID, Disconnect), Crash<()>>>> =
        thread::spawn(move || block_on(monitoring(&mut d1, future::pending::<io::Result<()>>())));

    assert_eq!((), t1.join().unwrap());

    let (i3, disco) = t2.join().unwrap().unwrap_err().unwrap();
    assert_eq!(disco, Disconnect::Crash);
    assert_eq!(i2 ,i3);
}
