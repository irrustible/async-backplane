use async_backplane::prelude::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

fn assert_disconnect(d: Device, fault: Option<Fault>) {
    assert_eq!((), spawn(move || d.disconnect(fault)).join().unwrap());
}

fn watch(mut d: Device) -> JoinHandle<Result<Watched<()>, Crash<()>>> {
    spawn(move || block_on(d.watch(ready(()))))
}

#[test]
fn solo_succeeds() {
    let d1 = Device::new();
    assert_eq!(Completed(()), watch(d1).join().unwrap().unwrap());
}

#[test]
fn monitored_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Monitor);
    assert_disconnect(d1, None);
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(device_id, did);
    assert_eq!(None, result);
}

#[test]
fn monitored_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link_line(d1.line(), LinkMode::Monitor).unwrap();
    assert_disconnect(d1, None);
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(device_id, did);
    assert_eq!(None, result);
}

#[test]
fn monitored_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Monitor);
    assert_disconnect(d1, Some(Fault::Error));
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn monitored_line_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).unwrap();
    assert_disconnect(d1, Some(Fault::Error));
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn monitored_device_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let device_id = d1.device_id();
        d2.link(&d1, LinkMode::Monitor);
        device_id
    };
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn monitored_line_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        let line = d1.line();
        d2.link_line(line, LinkMode::Monitor).unwrap();
        id
    };
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn peer_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    d2.link(&d1, LinkMode::Peer);
    let device_id = d1.device_id();
    assert_disconnect(d1, None);
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, None);
}

#[test]
fn peer_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link_line(d1.line(), LinkMode::Peer).unwrap();
    assert_disconnect(d1, None);
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, None);
}

#[test]
fn peer_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Peer);
    assert_disconnect(d1, Some(Fault::Error));
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn peer_line_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).unwrap();
    assert_disconnect(d1, Some(Fault::Error));
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Error));
}

#[test]
fn peer_device_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let device_id = d1.device_id();
        d2.link(&d1, LinkMode::Peer);
        device_id
    };
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}

#[test]
fn peer_line_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        d2.link_line(d1.line(), LinkMode::Peer).unwrap();
        d1.device_id()
    };
    let (did, result) = watch(d2).join().unwrap().unwrap()
        .unwrap_messaged().unwrap_disconnected();
    assert_eq!(did, device_id);
    assert_eq!(result, Some(Fault::Drop));
}
