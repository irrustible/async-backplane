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

Everything is believed to work correctly, but we're still too new to
be sure. The API may change slightly before 1.0, but nothing major, I
hope.

Please note that this is a more general purpose, lower-level tool than
most libraries that claim to be inspired by erlang. It is the plan
that other libraries will provide a higher level experience. I'm
working on some, which will be coming soon:

* [async-supervisor](https://github.com/irrustible/async-supervisor)

## Guide

### Introduction

The Backplane (that's a fancy word for 'motherboard') is a dynamic
mesh of `Device`s - owned objects representing a backplane
presence. On dropping a `Device` or calling its `disconnect()` method,
other Devices that have chosen to hear about it will be notified.

All erlang-style reliability springs from this one capability to be
notified of the failure of your dependencies. It is the lower-level
tool upon which more advanced concepts such as the famous supervisors
are built.

Creating a `Device` is easy:

```rust
use async_backplane::Device;

fn device() -> Device { Device::new() }
```

What is a `Device`? What does having a presence in the backplane mean?

* We maintain a list of `Devices` to notify.
* When we `disconnect`, we will notify those Devices.

There are two triggers for a disconnect:

* The `Device` is dropped.
* The `Device`'s `disconnect()` method is called.

Once a `Device` has disconnected, you can no longer use it. No more
linking, no more messaging, it is done.

The `Device` is a futures `Stream` and can be polled for `Message`s. A
message is one of two things:

* A request to shut down with the `DeviceID` of the requestor.
* A notification that another `Device` has disconnected. This contains
  the `DeviceID` of the disconnecting Device and an `Option<Fault>`
  describing the nature of the disconnect.
  
Here's an example of polling it in an async fn:

```rust
use async_backplane::{Device, Message};
use futures_lite::StreamExt; // for `.next()` on Stream

async fn next_message(device: &mut Device) -> Option<Message> {
    device.next().await
}
```

This is much more useful if there is something to listen for, which is
where linking comes in!

### Linking

Linking is how we configure Devices to notify each other when they
disconnect (drop or have `.disconnect()` called on them). There are
three types of link mode (`LinkMode`):

* `Monitor` - be notified when the other Device disconnects.
* `Notify` - notify the other Device when this Device disconnects.
* `Peer` - both notify each other when they disconnect.

Linking is pretty easy if you have a pair of Devices (such as when
you're spawning a new Device):

```rust
use async_backplane::Device;

// `l` will be notified when `r` disconnects
fn monitor(l: &Device, r: &Device) { l.link(r, LinkMode::Monitor); }

// `r` will be notified when `l` disconnects
fn notify(l: &Device, r: &Device) { l.link(r, LinkMode::Notify); }

// `l` will be notified when `r` disconnects
// `r` will be notified when `l` disconnects
fn peer(l: &Device, r: &Device) { l.link(r, LinkMode::Peer); }
```

Now we have something to listen for, let's keep restarting a failing
task for all eternity:

```rust

use async_backplane::*;
use futures_lite::StreamExt; // for `.next()` on Stream
use smol::Task; // just a small and simple futures executor

async fn never_stop<F: Fn(Device)>(mut device: Device, spawn: F) {
    loop { /// We want to go forever
        let d = Device::new();
        device.link(&d, LinkMode::Monitor);
        spawn(d);
        while let Some(message) = device.next().await {
            match message {
                Message::Shutdown(id) => (), // ignore!
                Message::Disconnected(_id, _fault) => { break; } // restart!
            }
        }
    }
}

/// This is quite obviously not going to succeed. Maybe yours should!
fn failing_task(device: Device) {
    smol::Task::spawn(async {
      device.disconnect(Some(Fault::Error(())))
    }).detach();
}

fn main() {
    never_stop(failing_task)
}
```

In a sense, we have just written our first supervisor! A new crate,
[async-supervisor](https://github.com/irrustible/async-supervisor)
is coming soon with erlang-style supervisors.

### Managed devices

Exciting as all this low level control over how we respond to exits
is, if we take the erlang model seriously, we generally leave this to
supervisors, and most of our tasks are *not* supervisors.

Non-supervisor tasks just want to get on with their work. That means
if any Device they are monitoring disconnects with a `Fault`, they too
will want to disconnect with a `Fault`. In this sense, links are a
*dependency graph* between `Device`s (which are proxies for the
computations using those `Device`s).

We call this extremely common scenario *managed mode*. It can be
accessed through the `Device.manage()` method:

```rust
use async_backplane::*;
use smol::Task;

fn example() {
    let device = Device::new();
    Task::spawn(async move {
        device.manage(async { Ok(()) }); // Succeed!
    }).detach();
}
```

There are three logical steps here:
* Creating the Device (`Device::new()`).
* Spawning a Future on the executor (`Task::spawn(...).detach()`).
* In the spawned Future, putting the Device into managed mode
  with an async block to execute (`device.manage(async { Ok(()) })`).

The async block you provide to `Device.manage()` should return a
`Result` of some kind. If you return `Ok`, the Device will be
considered to have completed without fault. If you return `Err`, the
Device will be considered to have faulted.

Managed devices will run until the first of:
* The provided future/async block returning a result.
* The provided future/async block unwind panicking.
* A Device sending us a message:
  * On receiving a shutdown request, complete successfully.
  * On receiving a disconnect notification that is fatal, fault.

By calling `.manage()`, you are giving up ownership of the Device
permanently. When one of the above happens, any Devices that are
monitoring us will be notified.

The `manage()` method returns a `Result<T, Crash<C>>` where `T` and
`C` are the success and error types of the `Result<T,C>` returned by
the async block. `Crash` is just an enum with an arm for each kind of
failure. It contains detailed information about what went wrong, whereas
any *notification* of our disconnection contains only basic information.

I'm still trying to work out what to do with crashes. I don't want
this library to be too opinionated or to bloat the dependency tree too
much. Maybe I'll do an opinionated library that uses this one, or
maybe you'll just create your own `manage_panic()` function in each
project and use that? Suggestions gratefully received!

### Dynamic link topologies

Often, we will want to use `Device.manage()` to get the automatic
management behaviour, but we'll also want to link with new Devices as
part of that work But `manage()` takes ownership of the `Device`
permanently, so what do we do?

A `Line` is a cloneable reference to a `Device` in the style of an
`Arc` (and indeed, contains one). The gotcha is that because the
`Line` is non-owning, the `Device` it references could have
disconnected by the time you try to use it, so linking may fail:

```rust
use async_backplane::*;

fn example() {
    let a = Device::new();
    let b = Device::new();
    let line = b.line();
    a.link_line(line, LinkMode::Monitor) // suspiciously like `.link()`...
      .unwrap(); // b clearly did not disconnect yet
    // ... spawn both ...
}
```

Note that `link_line()` consumes the `Line`. This is because
internally, the list of notifiable `Device`s is actually a list of
`Line`, so we avoid a clone in the case you no longer  need the `Line`.

You can link between `Lines` directly as well, since `Line` also has a
`link_line()` method:

```rust
use async_backplane::*;

fn demo() {
    let a = Device::new();
    let b = Device::new();
    let c = Device::new();
    let c2 = c.line();
    let d = Device::new();
    let d2 = d.line();
    a.link(&b, LinkMode::Peer); // Device-Device link
    b.link_line(c2, LinkMode::Peer).unwrap(); // Device-Line link
    c2.link_line(d2, LinkMode::Peer).unwrap(); // Line-Line link
    // ... now go spawn them all ...
}
```

Any time you will want dynamically link while you are using
`Device.manage()`, you should create a `Line` first.

#### A note of caution on dynamic topologies

Once you have linked with something through a `Line`, you should only
unlink it through the `Line`. Device-to-Device linkage is fast because
it avoids the work that would make it handle this case correctly. In
general, you should only link or unlink with `Device`s when you know
you have not previously linked with the corresponding `Line`s.

### Differences from Erlang/OTP

While I am very heavily inspired by Erlang and the OTP principles,
there's a bit of an impedance mismatch Rust and Erlang, in particular
when it comes to ownership versus garbage collection. backplane is
thus an adaptation of the principles that "feels right" for Rust.

Where it's ended up after a few months of R+D is as a lower level tool
that tries not to be too pushy and opinionated and is extremely small.

Here are some of the more striking differences

#### Separation between Device and logic

In erlang, when you wish to spawn a process, you provide a 0-arity
function. By default, it works essentially like `Device.manage()`
without the transfer of ownership.

In backplane, I do not want to force my choice of executor or
execution policy on you, so creating a `Device` is totally independent
of spawning the Future that will use it, out of necessity.

This means that while most code will called `Device.manage()`, you
have full freedom to implement whatever logic you want and to store
the `Device` where you want.

#### Separation between Device and Mailbox

In erlang, all messages sent to a process go through the same channel
(the mailbox). In a sense, a Device does have a mailbox, but it is of
strictly limited utility. `Device`s do not handle any messages other
than `Message`, whereas erlang messages may be anything. In order to
exchange general messages with the tasks using the `Device`s, you
would need to e.g. open an `async-channel` channel.

### FAQ

#### Why erlang?

Your author has been an Elixir programmer by profession for the last
few years and has come to appreciate deeply the principles underlying
the reliability of Erlang, upon which Elixir is based. Above all, what
I value is the simplicity. The entire system is simple enough to be
able to reason about at scale.

#### Haven't other people already tried this? Why reinvent the wheel?

Much of it is taste. I don't think any existing solutions really
capture the essence of what erlang reliability is about, or give a
feel for its essential beauty. People seem to get too tied up in
actors and supervision and focus less on the fundamentals.

Existing solutions also tend to be large, complex things that are
difficult to learn and reason out and pull in a lot of
dependencies. The whole point of erlang to me is that it makes
concurrency and dependency so simple, you can reason about them at
scale. But I fear we're drifting back to discussing taste.

I also gave [a specific comparison with
bastion](https://www.reddit.com/r/rust/comments/i1was2/asyncbackplane_simple_erlanginspired/g02ztn0/)
on reddit by request. Just my opinion, others are available.

### Library pairing recommendations

These work great alongside `async-backplane`:

* [async-oneshot](https://github.com/irrustible/async-oneshot) - a
  fast, small, full-featured, no-std compatible oneshot channel
  library.
* [async-oneshot-local](https://github.com/irrustible/async-oneshot) -
  the single-threaded partner to `async-oneshot`.
* [async-channel](https://github.com/stjepang/async-channel/) - great
  all-purpose async-aware MPMC channel.
* [smol](https://github.com/stjepang/smol/) - small, high-performance
  multithreaded futures executor.
  
These will, when they're finished:

* [async-supervisor](https://github.com/irrustible/async-supervisor) -
  erlang-style supervisors for async-backplane.

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

If performance really matters, always link Device to Device. Also
spend some time optimising this library, because we didn't yet.

## Forthcoming work

* no_std support.
* Actors. Maybe.

## Copyright and License

Copyright (c) 2020 James Laver, async-backplane Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

