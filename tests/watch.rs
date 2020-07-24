use async_backplane::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

#[test]
fn solo_succeeds() {

    let mut d1 = Device::new();

    let t1: JoinHandle<Watching<()>> = spawn(move || block_on(d1.watch(ready(()))));

    // There isn't anything to fail, so it should succeed
    assert_eq!((), t1.join().unwrap().expect("success"));
}

// monitored via monitor

#[test]
fn monitored_succeeds() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let i1 = d1.device_id();
    let l1 = d1.line();
    d2.monitor(l1).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.completed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_complete());
    assert_eq!(i1 ,i3);
}

#[test]
fn monitored_crashes() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let i1 = d1.device_id();
    let l1 = d1.line();
    d2.monitor(l1).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.crashed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(i1 ,i3);
}

#[test]
fn monitored_drops() {
    let mut d2 = Device::new();
    let i1 = {
        let d1 = Device::new();
        let i1 = d1.device_id();
        let l1 = d1.line();
        d2.monitor(l1).expect("failed monitoring");
        i1
    };
    let t: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));

    let (i3, disco) = t.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(i1 ,i3);
}

// monitored via attach

#[test]
fn monitored_succeeds_attach() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let i1 = d1.device_id();
    let l2 = d2.line();
    d1.attach(l2).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.completed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_complete());
    assert_eq!(i1 ,i3);
}

#[test]
fn monitored_crashes_attach() {
    let d1 = Device::new();
    let mut d2 = Device::new();
    let i1 = d1.device_id();
    let l2 = d2.line();
    d1.attach(l2).expect("failed monitoring");
    let t1 = spawn(move || block_on(d1.crashed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());
    //We should hear about the complete first.
    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(i1 ,i3);
}

#[test]
fn monitored_drops_attach() {
    let mut d2 = Device::new();
    let i1 = {
        let d1 = Device::new();
        let i1 = d1.device_id();
        let l2 = d2.line();
        d1.attach(l2).expect("failed monitoring");
        i1
    };
    let t: JoinHandle<Watching<()>> = spawn(move || block_on(d2.watch(pending::<()>())));

    let (i3, disco) = t.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(i1 ,i3);
}

// linked

#[test]
fn linked_succeeds() {

    let mut d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();
    let l2 = d2.line();
    d1.link(l2).expect("failed linking");
    
    let t1 = spawn(move || block_on(d2.completed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d1.watch(pending::<()>())));
    assert_eq!((), t1.join().unwrap());

    // We should hear about the complete first.
    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_complete());
    assert_eq!(i2 ,i3);

}

#[test]
fn linked_crashes() {

    let mut d1 = Device::new();
    let d2 = Device::new();
    let i2 = d2.device_id();

    d1.link(d2.line()).expect("failed linking");

    let t1 = spawn(move || block_on(d2.crashed()));
    let t2: JoinHandle<Watching<()>> = spawn(move || block_on(d1.watch(pending::<()>())));

    assert_eq!((), t1.join().unwrap());

    let (i3, disco) = t2.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(i2 ,i3);
}

#[test]
fn linked_drops() {

    let mut d1 = Device::new();
    let did = { // d2 won't survive this block
        let d2 = Device::new();
        d1.link(d2.line()).expect("failed linking");
        d2.device_id()
    };

    let t: JoinHandle<Watching<()>> = spawn(move || block_on(d1.watch(pending::<()>())));

    let (did2, disco) = t.join().unwrap().unwrap_err().expect("disconnect notification");
    assert!(disco.is_crash());
    assert_eq!(did, did2);
}
