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

// create two devices, monitor one
#[bench]
fn monitor_drop(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let mut d2 = Device::new();
        d1.link(&mut d2, LinkMode::Monitor);
        black_box(d1);
        black_box(d2);
    })
}

// create two devices, monitor one
#[bench]
fn monitor_drop_line(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Monitor).unwrap();
        black_box(d1);
        black_box(d2);
    })
}

// drop in reverse order
#[bench]
fn monitor_drop_(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let mut d2 = Device::new();
        d1.link(&mut d2, LinkMode::Monitor);
        black_box(d2);
        black_box(d1);
    })
}

// drop in reverse order
#[bench]
fn monitor_drop_line_(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Monitor).unwrap();
        black_box(d2);
        black_box(d1);
    })
}

// create two devices, link them
#[bench]
fn peer_drop(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let mut d2 = Device::new();
        d1.link(&mut d2, LinkMode::Peer);
        black_box(d1);
        black_box(d2);
    })
}

// create two devices, link them
#[bench]
fn peer_drop_line(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Peer).unwrap();
        black_box(d1);
        black_box(d2);
    })
}

// drop in reverse order
#[bench]
fn peer_drop_(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let mut d2 = Device::new();
        d1.link(&mut d2, LinkMode::Peer);
        black_box(d2);
        black_box(d1);
    })
}

// drop in reverse order
#[bench]
fn peer_drop_line_(b: &mut Bencher) {
    b.iter(|| {
        let mut d1 = Device::new();
        let d2 = Device::new();
        let line = d2.line();
        d1.link_line(line, LinkMode::Peer).unwrap();
        black_box(d2);
        black_box(d1);
    })
}
