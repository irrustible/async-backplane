use async_backplane::*;
use futures_lite::future::{pending, ready, block_on};
use std::thread::{spawn, JoinHandle};

#[test]
fn solo_succeeds() {
    let d = Device::new();
    let t: JoinHandle<Manage<()>> =
        spawn(move || block_on(d.manage(ready(Ok(())))));
    assert_eq!((), t.join().unwrap().expect("success"));
}

#[test]
fn monitored_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    d2.link(&d1, LinkMode::Monitor);

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(ready(Ok(())))));
    assert_eq!((), t2.join().unwrap().expect("success"));
}

#[test]
fn monitored_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("link");

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(ready(Ok(())))));
    assert_eq!((), t2.join().unwrap().expect("success"));
}


#[test]
fn monitored_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Monitor);
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(report.device_id, device_id);
        assert!(report.result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_line_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Monitor).expect("link");
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(report.device_id, device_id);
        assert!(report.result.is_error());
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
    let t: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(device_id, report.device_id);
        assert_eq!(report.result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn monitored_line_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let line = d1.line();
        let id = d1.device_id();
        d2.link_line(line, LinkMode::Monitor).expect("link");
        id
    };
    let t: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(device_id, report.device_id);
        assert_eq!(report.result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn peer_device_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    d2.link(&d1, LinkMode::Peer);

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(ready(Ok(())))));
    assert_eq!((), t2.join().unwrap().expect("success"));
}

#[test]
fn peer_line_succeeds() {
    let d1 = Device::new();
    let d2 = Device::new();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).expect("link");

    let t1 = spawn(move || d1.disconnect(None));
    assert_eq!((), t1.join().unwrap());

    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(ready(Ok(())))));
    assert_eq!((), t2.join().unwrap().expect("success"));
}


#[test]
fn peer_device_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    d2.link(&d1, LinkMode::Peer);
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(report.device_id, device_id);
        assert!(report.result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn peer_line_crashes() {
    let d1 = Device::new();
    let d2 = Device::new();
    let device_id = d1.device_id();
    let line = d1.line();
    d2.link_line(line, LinkMode::Peer).expect("link");
    let t1 = spawn(move || d1.disconnect(Some(Fault::Error)));
    let t2: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    assert_eq!((), t1.join().unwrap());
    let crash = t2.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(report.device_id, device_id);
        assert!(report.result.is_error());
    } else {
        unreachable!();
    }
}

#[test]
fn peer_device_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let id = d1.device_id();
        d2.link(&d1, LinkMode::Peer);
        id
    };
    let t: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(device_id, report.device_id);
        assert_eq!(report.result, Fault::Drop);
    } else {
        unreachable!();
    }
}

#[test]
fn peer_link_drops() {
    let d2 = Device::new();
    let device_id = {
        let d1 = Device::new();
        let line = d1.line();
        let id = d1.device_id();
        d2.link_line(line, LinkMode::Peer).expect("link");
        id
    };
    let t: JoinHandle<Manage<()>> =
        spawn(move || block_on(d2.manage(pending())));
    let crash = t.join().unwrap().unwrap_err();
    if let Crash::Cascade(report) = crash {
        assert_eq!(device_id, report.device_id);
        assert_eq!(report.result, Fault::Drop);
    } else {
        unreachable!();
    }
}

