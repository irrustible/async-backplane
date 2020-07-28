#![feature(test)]

extern crate test;
use test::{black_box, Bencher};

use async_backplane::*;

#[bench]
fn create_destroy(b: &mut Bencher) {
    b.iter(|| {
        let d = Device::new();
        black_box(d);
    })
}

#[bench]
fn device_monitor_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Monitor);
        black_box(d1);
        black_box(d2);
    })
}

#[bench]
fn line_monitor_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Monitor).unwrap();
        black_box(d1);
        black_box(d2);
    })
}

#[bench]
fn device_monitor_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Monitor);
        black_box(d2);
        black_box(d1);
    })
}

#[bench]
fn device_monitor_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Monitor);
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}

#[bench]
fn line_monitor_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Monitor).unwrap();
        black_box(d2);
        black_box(d1);
    })
}

#[bench]
fn line_monitor_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Monitor).unwrap();
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}

#[bench]
fn device_peer_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Peer);
        black_box(d1);
        black_box(d2);
    })
}

#[bench]
fn device_peer_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(&d2, LinkMode::Peer);
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}

#[bench]
fn line_peer_drop_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Peer).unwrap();
        black_box(d1);
        black_box(d2);
    })
}

#[bench]
fn line_peer_error_notify(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Peer).unwrap();
        d2.disconnect(Some(Fault::Error));
        black_box(d1);
    })
}
