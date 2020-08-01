# async-backplane

[![License](https://img.shields.io/crates/l/async-backplane.svg)](https://github.com/irrustible/async-backplane/blob/main/LICENSE)
[![Package](https://img.shields.io/crates/v/async-backplane.svg)](https://crates.io/crates/async-backplane)
[![Documentation](https://docs.rs/async-backplane/badge.svg)](https://docs.rs/async-backplane)

Easy, Erlang-inspired fault-tolerance framework for Rust Futures.

Features:

* The secrets of Erlang's legendary reliability.
* Idiomatic Rust API with low-level control.
* Simple. Easy to learn and use.
* Plays nicely with the existing Futures Ecosystem
* Uses no unstable features or unsafe code.
* High performance and (relatively) low memory
* Lightweight: ~600 lines of code, 6 deps, fresh build in seconds. 
* No `Box<dyn Any>`, LOL.

## Status.

Beta quality. Everything appears to work correctly, but we want to
write more tests before we feel confident it is correct. I have fixed
little bugs as I've noticed them, so clearly we needed better tests.

The API may change slightly before 1.0, but nothing major, I
hope. Broadly speaking, I'm delighted with it and any changes are
likely to be small, when I discover things I can't do with it.

## Guide

The Backplane (that's a fancy word for 'motherboard') is a dynamic
mesh of `Device`s. The `Device` object is a Future's connection into
the Backplane. It maintains connections to other Devices, such that
when we disconnect (complete), we notify them. We can connect to
another device with `Device.link()`, passing a `LinkMode`, of which
there are three:

* `Monitor` - be notified when the other Device disconnects.
* `Notify` - notify the other Device when this Device disconnects.
* `Peer` - both notify each other when they disconnect.

The way we react to these disconnections is what makes our
applications reliable. Erlang's equivalent of a spawned future, a
`process`, is categorised according to how they handle errors:

* `worker` processes notified of a failure will fail themselves
* `supervisor` processes notified of a completion will apply some sort
  of logic to restart processes under their supervision.

In async-backplane, `worker` corresponds to the `Device.manage()`
method. Here's an example using the 'smol' futures executor:

```rust
use async_backplane::*;
use smol::Task;

fn example() {
    let device = Device::new();
    Task::spawn(async move {
        device.manage(async { ... });
    }).detach();
}
```

There are three logical steps here:
* Creating the Device (`Device::new()`).
* Spawning a Future on the executor (`Task::spawn(...).detach()`).
* In the spawned Future, putting the Device into managed mode
  with an async block to execute (`device.manage(async { ... }`)`

Managed devices will run until the first of:
* The async block returning a result.
* The async block unwind panicking.
* A Device sending us a message:
  * On receiving a shutdown request, complete successfully.
  * On receiving a disconnect notification that is fatal, fault.

The async block you provide should return a `Result` of some kind. If
you return `Ok`, the Device will be considered to have successfully
completed its work. If you return `Err`, the Device will be considered
to have faulted.

When any of these conditions has occurred, the Device will notify all
Devices which are monitoring us of our status and the Device will be
dropped. The `manage()` method returns a `Result<T, Crash<C>>` where T
is the success type of the Result returned by the async block. C is
the error type for the same Result returned by the async
block. `Crash` is just an enum with an arm for each kind of failure.

I'm still trying to work out what to do with crashes. I don't want
this library to be too opinionated or to bloat the dependency tree too
much. Maybe I'll do an opinionated library that uses this one, or
maybe you'll just create your own `manage_panic()` function in each
project and use that? Suggestions gratefully received!

### Recovery

`Device.watch()` is the tool for building more complex behaviours. It
protects against unwind panics and listens for disconnects, but it
just returns the first of the provided future's result and the next
disconnect to occur.

One of the more useful things you can do with watch is recreate
futures that have failed. Indeed, this is how supervisors work, both
in erlang and the library i'm currently building,
[async-supervisors](https://github.com/irrustible/async-supervisors)

### Static link topologies

Devices can be linked together by calling their `link()` method. They
take a `LinkMode` as described back at the start of the guide. Example:

```rust
use async_backplane::*;

fn demo() {
    let a = Device::new();
    let b = Device::new();
    let c = Device::new();
    a.link(&b, LinkMode::Peer);
    b.link(&c, LinkMode::Peer);
    // ... now go spawn them all ...
}
```

### Dynamic link topologies

Most of our Devices will be running in managed mode after they have
been set up. Managed mode takes ownership of our `Device`, so how do
we link? Enter the `Line`, a reference to a `Device` that can be
cloned and passed around freely.

Getting a `Line` is simple: `device.line()`. Linking to a `Line` from
a `Device` is much like linking to a `Device`, except we call
`link_line()` instead of `link()`. Unlike `link()`:

* It consumes the provided Line (to avoid an unnecessary clone)
* It may fail because the `Device` the line is connected to has
  disconnected, so it returns a `Result`.

You can link between `Lines` directly as well: `Line` also has a
`link_line()` method!

```rust
use async_backplane::*;

fn demo() {
    let a = Device::new();
    let b = Device::new();
    let c = Device::new();
    let c2 = c.line();
    let d = Device::new();
    let d2 = d.line();
    a.link(&b, LinkMode::Peer);
    b.link_line(c2, LinkMode::Peer).unwrap();
    c2.link_line(d2, LinkMode::Peer).unwrap();
    // ... now go spawn them all ...
}
```

#### A note of caution on mixed topologies

Once you have linked with something through a `Line`, you should only
unlink it through the `Line`. Device-to-Device linkage is fast because
it avoids the work that would make it handle this case correctly. In
general, you should only link or unlink with `Device`s when you know
you have not previously linked with the corresponding `Line`s.

## Relationship to Erlang/OTP

async-backplane does not implement actors, only links and monitors. It
is a lower level tool that allows for a wider range of usage
patterns. You could build actors (and other things!) on top of this. 

## Library pairing recommendations

These work great alongside `async-backplane`:

* [async-channel](https://github.com/stjepang/async-channel/) - great
  all-purpose async-aware channel.
* [smol](https://github.com/stjepang/smol/) - small, high-performance
  multithreaded futures executor.

## Forthcoming work

Note: these will likely be new libraries, linked from here when public.

* Supervisors: [async-supervisors](https://github.com/irrustible/async-supervisors)
* Actors.
* no_std support.

## Performance

These numbers are random unscientific benchmark measurements from my
shitty 2015 macbook pro. Your numbers may be different. Run the
benchmarks, or better still, bench your real world code using it.

```
     Running target/release/deps/device-8add01b9803770b5

running 11 tests
test create_destroy              ... bench:         212 ns/iter (+/- 9)
test device_monitor_drop         ... bench:         585 ns/iter (+/- 64)
test device_monitor_drop_notify  ... bench:         771 ns/iter (+/- 39)
test device_monitor_error_notify ... bench:         798 ns/iter (+/- 39)
test device_peer_drop_notify     ... bench:         964 ns/iter (+/- 40)
test device_peer_error_notify    ... bench:         941 ns/iter (+/- 304)
test line_monitor_drop           ... bench:         805 ns/iter (+/- 48)
test line_monitor_drop_notify    ... bench:         975 ns/iter (+/- 48)
test line_monitor_error_notify   ... bench:         993 ns/iter (+/- 55)
test line_peer_drop_notify       ... bench:       1,090 ns/iter (+/- 62)
test line_peer_error_notify      ... bench:       1,181 ns/iter (+/- 65)

test result: ok. 0 passed; 0 failed; 0 ignored; 11 measured; 0 filtered out

     Running target/release/deps/line-c87021ef05fddd66

running 6 tests
test create_destroy            ... bench:          13 ns/iter (+/- 4)
test line_monitor_drop         ... bench:         793 ns/iter (+/- 51)
test line_monitor_drop_notify  ... bench:         968 ns/iter (+/- 357)
test line_monitor_error_notify ... bench:       1,018 ns/iter (+/- 54)
test line_peer_drop_notify     ... bench:       1,343 ns/iter (+/- 70)
test line_peer_error_notify    ... bench:       1,370 ns/iter (+/- 77)
```

Note that when linking, it is cheaper to use a Device than a Line, that is:

* `device.link()` is fastest.
* `device.link_line()` is slightly more expensive.
* `line.link_line()` is slightly more expensive still.

If performance really matters, do not use dynamic topologies. Also
spend some time microoptimising this library, because we didn't yet.

## Copyright and License

Copyright (c) 2020 James Laver, async-backplane Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

