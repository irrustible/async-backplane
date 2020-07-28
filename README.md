# async-backplane

<!-- [![License](https://img.shields.io/crates/l/async-backplane.svg)](https://github.com/irrustible/async-backplane/blob/main/LICENSE) -->
<!-- [![Package](https://img.shields.io/crates/v/async-backplane.svg)](https://crates.io/crates/async-backplane) -->
<!-- [![Documentation](https://docs.rs/async-backplane/badge.svg)](https://docs.rs/async-backplane) -->

Simple, Erlang-inspired reliability backplane for Rust Futures.

## Status.

Beta. We're delighted with it, but we're still testing and polishing.

## Overview

A Future that wishes to participate in the backplane creates a
`Device`, which may be linked to other devices to exchange termination
information. There are three types of link:

* Monitor - be notified when the other Device terminates.
* Notify - notify the other Device when this Device terminates.
* Peer - equivalent to Monitor + Notify.

A Device may terminate successfully or with an error. Monitoring
devices may react to an error by themselves terminating with an error
or they may choose to take corrective action. This is the basis for
building reliable applications with async_backplane.

## Example usage

```rust
use async_backplane::*;
use futures_lite::future::pending;

fn main() {
    let d1 = Device::new();
    d1.spawn(|d1| {
        async { // We will restart a child until it succeeds
            loop {
                let d2 = Device::new();
                // d1 will hear about d2's termination
                d1.link(&d2, LinkMode::Monitor);
                d2.spawn(|d| async {// spawns on the executor, requires smol
                    d.manage(|| {
                        // your code goes here...
                        Err(()) // This will cause the device to fail
                    }).await;
                });
                ///
                match d.watch(|| pending(())).await {
                    Ok(Or::Left(())) => { return Ok(()); }
                    Ok(Or::Right(report)) => {
                        if !report.inner.is_some() {
                            return Ok(()); // Done!
                        }
                    }
                    /// This is *us* panicking. Can't see how...
                    Err(crash) => { return Err(crash); }
                }
            }
        }
    });
}
```

## Reliability patterns

There are three methods on `Device` that help you to build reliable
applications. All of them:

* Wrap the provided future such that it catches unwind panics.
* Polls the device for disconnections of linked devices.

The most useful one of these is `manage`. It consumes a Device,
wrapping a provided future to:

* Catch unwind panics during execution and promotes them into faults.
* Promote returning `Ok()` into a success.
* Promote returning `Err()` into a fault.
* Listen for disconnects from monitored devices:
  * If successful, remove it from our monitors list if present.
  * If it faulted, cascade the fault (fault in sympathy).
* Notifies monitors of our disconnect with our success/fault status.

`part_manage` is a temporary version of `manage`. It will not notify
monitors in the event of returning `Ok()` as it is assumed you will
wish to continue with the Device. In in case of fault, still notifies.

`watch` is for building more complex behaviours. It protects against
unwind panics and monitors other devices for failure, but it just
returns the first of the provided future and the next disconnect to
occur.

One of the more useful things you can do with watch is recreate
futures that have failed. Be sure to link appropriately!

## Relationship to Erlang/OTP

async-backplane does not implement actors, only links and monitors. It
is a lower level tool that allows for a wider range of usage
patterns. You could build actors (and other things!) on top of this. I
will be doing that very soon.

## Performance

These numbers are random unscientific benchmark measurements from my
shitty 2015 macbook pro. Your numbers may be different. Run the
benchmarks, or better still bench your real world code using it.

```
running 11 tests
test create_destroy              ... bench:         203 ns/iter (+/- 35)
test device_monitor_drop         ... bench:         524 ns/iter (+/- 52)
test device_monitor_drop_notify  ... bench:         698 ns/iter (+/- 87)
test device_monitor_error_notify ... bench:         726 ns/iter (+/- 63)
test device_peer_drop_notify     ... bench:         897 ns/iter (+/- 130)
test device_peer_error_notify    ... bench:         948 ns/iter (+/- 147)
test line_monitor_drop           ... bench:         775 ns/iter (+/- 161)
test line_monitor_drop_notify    ... bench:         914 ns/iter (+/- 80)
test line_monitor_error_notify   ... bench:         947 ns/iter (+/- 133)
test line_peer_drop_notify       ... bench:       1,037 ns/iter (+/- 151)
test line_peer_error_notify      ... bench:       1,083 ns/iter (+/- 132)

test result: ok. 0 passed; 0 failed; 0 ignored; 11 measured; 0 filtered out

     Running target/release/deps/line-3578157f35e6c856

running 6 tests
test create_destroy            ... bench:          13 ns/iter (+/- 2)
test line_monitor_drop         ... bench:         722 ns/iter (+/- 79)
test line_monitor_drop_notify  ... bench:         916 ns/iter (+/- 144)
test line_monitor_error_notify ... bench:         958 ns/iter (+/- 171)
test line_peer_drop_notify     ... bench:       1,225 ns/iter (+/- 168)
test line_peer_error_notify    ... bench:       1,238 ns/iter (+/- 174)

test result: ok. 0 passed; 0 failed; 0 ignored; 6 measured; 0 filtered out
```

Conclusions:

* We're pretty fast! Imagine how fast we'll be when it's optimised...
* Prefer Devices over Lines where speed is essential.

## Copyright and License

Copyright (c) 2020 James Laver, async-backplane Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

