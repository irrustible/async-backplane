#![feature(test)]

extern crate test;
use test::Bencher;

use async_backplane::*;

#[bench]
fn create_destroy(b: &mut Bencher) {
    b.iter(|| {
        let d = Device::new();
        test::black_box(d);
    })
}

// create two devices, monitor one
#[bench]
fn monitor_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.monitor(d2.line()).unwrap();
        test::black_box(d1);
        test::black_box(d2);
    })
}

// drop in reverse order
#[bench]
fn monitor_drop_(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.monitor(d2.line()).unwrap();
        test::black_box(d2);
        test::black_box(d1);
    })
}

// create two devices, attach one
#[bench]
fn attach_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.attach(d2.line()).unwrap();
        test::black_box(d1);
        test::black_box(d2);
    })
}

// drop in reverse order
#[bench]
fn attach_drop_(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.attach(d2.line()).unwrap();
        test::black_box(d2);
        test::black_box(d1);
    })
}

// create two devices, link them
#[bench]
fn link_drop(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(d2.line()).unwrap();
        test::black_box(d1);
        test::black_box(d2);
    })
}
// drop in reverse order
#[bench]
fn link_drop_(b: &mut Bencher) {
    b.iter(|| {
        let d1 = Device::new();
        let d2 = Device::new();
        d1.link(d2.line()).unwrap();
        test::black_box(d2);
        test::black_box(d1);
    })
}
