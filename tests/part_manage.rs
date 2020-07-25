use async_backplane::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

#[test]
fn solo_succeeds() {

    let d1 = Device::new();
    let did = d1.device_id();
    let t1: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d1.part_manage(ready(Ok(())))));
    let (d2, ret) = t1.join().unwrap().expect("success");
    assert_eq!(did, d2.device_id());
    assert_eq!(ret, ());
}

// monitored via monitor

#[test]
fn monitored_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let did = d2.device_id();
    let l1 = d1.line();
    d2.monitor(l1).expect("failed monitoring");

    let t1 = spawn(move || block_on(d1.completed()));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, ret) = t2.join().unwrap().expect("success");
    assert_eq!(ret, ());
    assert_eq!(d3.device_id(), did);
}

#[test]
fn monitored_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let i1 = d1.device_id();
    let l1 = d1.line();
    d2.monitor(l1).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.crashed()));
    let t2: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, disco) = crash {
        assert_eq!(i1, did);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_drops() {
    let d2 = Device::new();
    let i1 = {
        let d1 = Device::new();
        let i1 = d1.device_id();
        let l1 = d1.line();
        d2.monitor(l1).expect("failed monitoring");
        i1
    };
    let t: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, disco) = crash {
        assert_eq!(i1, did);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}

// monitored via attach

#[test]
fn attached_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let did = d2.device_id();
    let l2 = d2.line();
    d1.attach(l2).expect("failed monitoring");

    let t1 = spawn(move || block_on(d1.completed()));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, ret) = t2.join().unwrap().expect("success");
    assert_eq!(ret, ());
    assert_eq!(d3.device_id(), did);
}

#[test]
fn attached_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let i1 = d1.device_id();
    let l2 = d2.line();
    d1.attach(l2).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.crashed()));
    let t2: JoinHandle<PartManaging<()>> = spawn(move || block_on(d2.part_manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, disco) = crash {
        assert_eq!(i1, did);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}

#[test]
fn attached_drops() {
    let d2 = Device::new();
    let i1 = {
        let d1 = Device::new();
        let i1 = d1.device_id();
        let l2 = d2.line();
        d1.attach(l2).expect("failed monitoring");
        i1
    };
    let t: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, disco) = crash {
        assert_eq!(i1, did);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}

// linked

#[test]
fn linked_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let did = d2.device_id();
    let l2 = d2.line();
    d1.link(l2).expect("failed monitoring");

    let t1 = spawn(move || block_on(d1.completed()));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, ret) = t2.join().unwrap().expect("success");
    assert_eq!(ret, ());
    assert_eq!(d3.device_id(), did);
}

#[test]
fn linked_crashes() {

    let d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();

    d1.link(d2.line()).expect("failed linking");

    let t1 = spawn(move || block_on(d2.crashed()));
    let t2: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d1.part_manage(pending())));

    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, disco) = crash {
        assert_eq!(i2, did);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}

#[test]
fn linked_drops() {

    let d1 = Device::new();
    let did = { // d2 won't survive this block
        let d2 = Device::new();
        d1.link(d2.line()).expect("failed linking");
        d2.device_id()
    };

    let t: JoinHandle<PartManaging<()>> =
        spawn(move || block_on(d1.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did2, disco) = crash {
        assert_eq!(did, did2);
        assert!(disco.is_crash());
    } else {
        unreachable!();
    }
}
