use async_backplane::prelude::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

#[test]
fn solo_succeeds() {

    let mut d1 = Device::new();

    let t1: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d1.watch(ready(()))));
    // There isn't anything to fail, so it should succeed
    assert_eq!(Completed(()), t1.join().unwrap().expect("Success"));
}

#[test]
fn monitored_device_succeeds() {
    let mut d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&mut d1, LinkMode::Monitor);
    let t1 = spawn(move || d1.disconnect(None));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    let (did, result) = t2.join().unwrap().expect("success")
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(device_id, did);
    assert_eq!(None, result);
}

#[test]
fn monitored_line_succeeds() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("link");
    let t1 = spawn(move || d1.disconnect(None));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    let (did, result) = t2.join().unwrap().expect("success")
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(device_id, did);
    assert_eq!(None, result);
}

#[test]
fn monitored_device_crashes() {
    let mut d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&mut d1, LinkMode::Monitor);
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    let (did, result) = t2.join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn monitored_line_crashes() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("link");
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (did, result) = t2.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn monitored_device_drops() {
    let mut d2 = Device::new();
    let device_id = {
        let mut d1 = Device::new();
        let device_id = d1.device_id();
        d2.link(&mut d1, LinkMode::Monitor);
        device_id
    };
    let t: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));

    let (did, result) = t.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn monitored_line_drops() {
    let mut d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        let line = d1.line();
        d2.link_line(line, LinkMode::Monitor).expect("to link");
        id
    };
    let t: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));

    let (did, result) = t.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn peer_device_succeeds() {
    let mut d1 = Device::new();
    let mut d2 = Device::new();
    d2.link(&mut d1, LinkMode::Peer);
    let device_id = d1.device_id();
    let t1 = spawn(move || d1.disconnect(None));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (did, result) = t2.join().unwrap().expect("success").unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, None);
}

#[test]
fn peer_line_succeeds() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).expect("link");
    let t1 = spawn(move || d1.disconnect(None));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (did, result) = t2.join().unwrap().expect("success").unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, None);
}

#[test]
fn peer_device_crashes() {
    let mut d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&mut d1, LinkMode::Peer);
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    let (did, result) = t2.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn peer_line_crashes() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).expect("link");
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    let (did, result) = t2.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn peer_device_drops() {
    let mut d2 = Device::new();
    let device_id = {
        let mut d1 = Device::new();
        let device_id = d1.device_id();
        d2.link(&mut d1, LinkMode::Peer);
        device_id
    };
    let t: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));

    let (did, result) = t.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn peer_line_drops() {
    let mut d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        let line = d1.line();
        d2.link_line(line, LinkMode::Peer).expect("to link");
        id
    };
    let t: JoinHandle<Result<Watched<()>,Crash<()>>> =
        spawn(move || block_on(d2.watch(pending::<()>())));

    let (did, result) = t.join().unwrap().unwrap().unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}
