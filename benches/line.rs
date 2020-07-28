#![feature(test)]
extern crate test;
use test::{black_box, Bencher};

use async_backplane::*;

#[bench]
fn create_destroy(b: &mut Bencher) {
    let d = Device::new();
    b.iter(|| { black_box(d.line()) });
}

// create two devices, monitor one
#[bench]
fn line_monitor_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let l1 = d1.line();
        let l2 = d2.line();
        l1.link_line(l2, LinkMode::Monitor).unwrap();
        black_box(l1);
        black_box(d1);
        black_box(d2);
    })
}

#[bench]
fn line_monitor_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let l1 = d1.line();
        let l2 = d2.line();
        l1.link_line(l2, LinkMode::Monitor).unwrap();
        black_box(l1);
        black_box(d2);
        black_box(d1);
    })
}

#[bench]
fn line_monitor_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let l1 = d1.line();
        let l2 = d2.line();
        l1.link_line(l2, LinkMode::Monitor).unwrap();
        black_box(l1);
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}

#[bench]
fn line_peer_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let l1 = d1.line();
        let l2 = d2.line();
        l1.link_line(l2, LinkMode::Peer).unwrap();
        black_box(l1);
        black_box(d2);
        black_box(d1);
    })
}

#[bench]
fn line_peer_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let l1 = d1.line();
        let l2 = d2.line();
        l1.link_line(l2, LinkMode::Peer).unwrap();
        black_box(l1);
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}
