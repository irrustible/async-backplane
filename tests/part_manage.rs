use async_backplane::prelude::*;
use futures_lite::future::{pending, ready, block_on};
// use futures_lite::stream::StreamExt;
use std::thread::{spawn, JoinHandle};

fn assert_disconnect(d: Device, fault: Option<Fault>) {
    assert_eq!((), spawn(move || d.disconnect(fault)).join().unwrap());
}

fn fail(d: Device) -> JoinHandle<Result<(Device, ()), Crash<()>>> {
    spawn(move || block_on(d.part_manage(pending())))
}

fn succeed(d: Device) -> JoinHandle<Result<(Device, ()), Crash<()>>> {
    spawn(move || block_on(d.part_manage(ready(Ok(())))))
}

fn watch(mut d: Device) -> JoinHandle<Result<Watched<()>, Crash<()>>> {
    spawn(move || block_on(d.watch(ready(()))))
}

#[test]
fn solo_succeeds() {
    let d1 = Device::new();
    let i1 = d1.device_id();
    let (d2, result) = succeed(d1).join().unwrap().unwrap();
    assert_eq!(i1, d2.device_id());
    assert_eq!(result, ());
}

// monitored

#[test]
fn monitored_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d2.link(&d1, LinkMode::Monitor);
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, None);
    let (d4, result) = succeed(d2).join().unwrap().unwrap();
    assert_eq!(d4.device_id(), i2);
    assert_eq!(result, ());
    assert_eq!(Completed(()), watch(d3).join().unwrap().unwrap());
}

#[test]
fn monitored_device_errors() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i1 = d1.device_id();
    let i2 = d2.device_id();
    d2.link(&d1, LinkMode::Monitor);
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, Some(Fault::Error));
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(i1, did);
        assert_eq!(Fault::Error, result);
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn monitored_device_drops() {
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d3.link(&d2, LinkMode::Monitor);
    let i1 = {
        let d1 = Device::new();
        d2.link(&d1, LinkMode::Monitor);
        d1.device_id()
    };
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert_eq!(result, Fault::Drop);
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn peer_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d2.link(&d1, LinkMode::Peer);
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, None);
    let (d4, result) = succeed(d2).join().unwrap().unwrap();
    assert_eq!(d4.device_id(), i2);
    assert_eq!(result, ());
    assert_eq!(Completed(()), watch(d3).join().unwrap().unwrap());
}

#[test]
fn peer_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i1 = d1.device_id();
    let i2 = d2.device_id();
    d2.link(&d1, LinkMode::Peer);
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, Some(Fault::Error));
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert!(result.is_error());
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn peer_device_drops() {
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d3.link(&d2, LinkMode::Peer);
    let i1 = { // d1 won't survive this block
        let d1 = Device::new();
        d2.link(&d1, LinkMode::Peer);
        d1.device_id()
    };
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert_eq!(result, Fault::Drop);
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn monitored_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).unwrap();
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, None);
    let (d4, result) = succeed(d2).join().unwrap().unwrap();
    assert_eq!(d4.device_id(), i2);
    assert_eq!(result, ());
    assert_eq!(Completed(()), watch(d3).join().unwrap().unwrap());
}

#[test]
fn monitored_line_errors() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i1 = d1.device_id();
    let i2 = d2.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).unwrap();
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, Some(Fault::Error));
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert!(result.is_error());
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn monitored_line_drops() {
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d3.link(&d2, LinkMode::Peer);
    let i1 = {
        let d1 = Device::new();
        d2.link_line(d1.line(), LinkMode::Monitor).unwrap();
        d1.device_id()
    };
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert_eq!(result, Fault::Drop);
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn peer_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).unwrap();
    d3.link(&d2, LinkMode::Peer);
    assert_disconnect(d1, None);
    let (d4, result) = succeed(d2).join().unwrap().unwrap();
    assert_eq!(d4.device_id(), i2);
    assert_eq!(result, ());
    assert_eq!(Completed(()), watch(d3).join().unwrap().unwrap());
}

#[test]
fn peer_line_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let d3 = Device::new();
    let i1 = d1.device_id();
    let i2 = d2.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).unwrap();
    d3.link(&d2, LinkMode::Monitor);
    assert_disconnect(d1, Some(Fault::Error));
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert!(result.is_error());
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

#[test]
fn peer_line_drops() {
    let d2 = Device::new();
    let d3 = Device::new();
    let i2 = d2.device_id();
    d3.link(&d2, LinkMode::Peer);
    let i1 = { // d1 won't survive this block
        let d1 = Device::new();
        d2.link_line(d1.line(), LinkMode::Peer).unwrap();
        d1.device_id()
    };
    if let Crash::Cascade(did, result) = fail(d2).join().unwrap().unwrap_err() {
        assert_eq!(did, i1);
        assert_eq!(result, Fault::Drop);
    } else { panic!() }
    let r3 = watch(d3).join().unwrap().unwrap();
    assert_eq!(Messaged(Disconnected(i2, Some(Fault::Cascade(i1)))), r3); 
}

