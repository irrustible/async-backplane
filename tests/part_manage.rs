use async_backplane::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

#[test]
fn solo_succeeds() {

    let d1 = Device::new();
    let did = d1.device_id();
    let t1: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d1.part_manage(ready(Ok(())))));
    let (d2, ret) = t1.join().unwrap().expect("success");
    assert_eq!(did, d2.device_id());
    assert_eq!(ret, ());
}

// monitored

#[test]
fn monitored_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();
    d2.link(&d1, LinkMode::Monitor);

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, result) = t2.join().unwrap().expect("success");
    assert_eq!(d3.device_id(), device_id);
    assert_eq!(result, ());
}

#[test]
fn monitored_device_errors() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Monitor);
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(device_id, did);
        assert!(result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_device_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        d2.link(&d1, LinkMode::Monitor);
        id
    };
    let t: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert_eq!(result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn peer_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();
    d1.link(&d2, LinkMode::Peer);

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, result) = t2.join().unwrap().expect("success");
    assert_eq!(d3.device_id(), device_id);
    assert_eq!(result, ());
}


#[test]
fn peer_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();

    d1.link(&d2, LinkMode::Peer);

    let t1 = spawn(move || d2.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d1.part_manage(pending())));

    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert!(result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn peer_device_drops() {
    let d1 = Device::new();
    let device_id = { // d2 won't survive this block
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Peer);
        d2.device_id()
    };

    let t: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d1.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert_eq!(result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("to link successfully");

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, result) = t2.join().unwrap().expect("success");
    assert_eq!(d3.device_id(), device_id);
    assert_eq!(result, ());
}

#[test] // hangs forever - why?! TODO
fn monitored_line_errors() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("to link successfully");
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(device_id, did);
        assert!(result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_line_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        let line = d1.line();
        d2.link_line(line, LinkMode::Monitor).expect("to link successfully");
        id
    };
    let t: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert_eq!(result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn peer_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();
    let line = d2.line();
    d1.link_line(line, LinkMode::Peer).expect("to link successfully");

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d2.part_manage(ready(Ok(())))));
    let (d3, result) = t2.join().unwrap().expect("success");
    assert_eq!(d3.device_id(), device_id);
    assert_eq!(result, ());
}

#[test]
fn peer_line_crashes() {

    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d2.device_id();
    let line = d2.line();
    d1.link_line(line, LinkMode::Peer).expect("to link successfully");

    let t1 = spawn(move || d2.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d1.part_manage(pending())));

    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert!(result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn peer_line_drops() {

    let d1 = Device::new();
    let device_id = { // d2 won't survive this block
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Peer).expect("to link successfully");
        d2.device_id()
    };

    let t: JoinHandle<Result<(Device, ()), Crash<()>> =
        spawn(move || block_on(d1.part_manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(did, result) = crash {
        assert_eq!(did, device_id);
        assert_eq!(result, Fault::Drop);
    } else {
        unreachable!();
    }
}

