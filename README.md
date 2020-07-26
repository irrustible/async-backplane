# async-backplane

Simple, Erlang-inspired reliability backplane for Rust Futures.

## Status.

Beta. We're delighted with it, but we're still polishing.

## Overview

A future that wishes to participate in the backplane creates a
`Device`, which may be linked to other devices to exchange termination
information. There are three types of link:

* Monitor - be notified when the other Device terminates.
* Notify - notify the other Device when this Device terminates.
* Peer - equivalent to Monitor + Notify.

A Device may terminate successfully or with an error. Monitoring
devices may react to an error by themselves terminating with an error
or they may choose to take corrective action. This is the basis for
building reliable applications with async_backplane.

## Relationship to Erlang/OTP

async-backplane does not implement actors, only links and monitors. It
is a lower level tool that allows for a wider range of usage patterns.
I will be releasing an actors library in the near future.

## Performance

These numbers are random unscientific benchmark measurements from my
shitty 2015 macbook pro. Your numbers may vary. The benchmarks are in
the repo - have a run.

### Device

```
test create_destroy             ... bench:         207 ns/iter (+/- 16)
test device_monitor_drop        ... bench:         552 ns/iter (+/- 30)
test device_monitor_drop_notify ... bench:         734 ns/iter (+/- 149)
test device_peer_drop_notify    ... bench:         870 ns/iter (+/- 156)
test line_monitor_drop          ... bench:         753 ns/iter (+/- 71)
test line_monitor_drop_notify   ... bench:         929 ns/iter (+/- 157)
test line_peer_drop_notify      ... bench:       1,064 ns/iter (+/- 74)
```

* Most tests create/destroy two devices.
* `create_destroy` test only creates/destroys one Device.
* `device` tests link through a Device.
* `line` tests create a Line and link through it.
* `monitor` tests establish a one-way link.
* `peer` tests establish a 2-way link.
* `notify` tests send 2 notifications for `peer` tests and 1 for `monitor` tests.

### Line

```
test create_destroy           ... bench:          14 ns/iter (+/- 0)
test line_monitor_drop        ... bench:         805 ns/iter (+/- 114)
test line_monitor_drop_notify ... bench:         983 ns/iter (+/- 105)
test line_peer_drop_notify    ... bench:       1,306 ns/iter (+/- 161)
```

Notes:

* Most tests create/destroy two Devices and two Lines and link through lines.
* `create_destroy` test only creates/destroys a Line from an existing Device.
* `notify` tests send 2 notifications for the `peer` test and 1 for the `monitor` test.

Conclusions:

* We're pretty fast!
* Prefer Devices over Lines where speed is essential.

## Copyright and License

Copyright (c) 2020 James Laver, async-backplane Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

