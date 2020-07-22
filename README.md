# async-backplane

Simple, Erlang-inspired reliability backplane for Rust Futures.

## Status.

Alpha. Very new. We invented a ton of stuff that might not work properly. 

## Overview

async-backplane lets us model dependency between independent
Futures. Futures in the backplane may choose to connect to each other
such that when one of them completes, the other is notified. This
library implements the core mechanism for this, along with some
structures for building reliable applications.

A participating `Future` uses a `Device` to connect to the
backplane. Device A may `.monitor()` Device B through a `Line`
returned by calling `.open_line()` on it. In the event that Device B
disconnects, Device A is able to learn of this fact. Two Devices which
are both monitoring each other are said to be `linked`.

## Reliability

Coming Soon.

## Relationship to Erlang/OTP

async-backplane is in some ways a level lower tool than erlang. This
is deliberate - we want it to be useful in as many contexts as
possible, not just something of interest to people who like erlang.

When people think of erlang, they think of the actor model and
supervisors. Both of these are very cool things, but they're not the
essence of what erlang is about. Erlang is all about the links and monitors.

The entirety of OTP's reliability is built upon links and
monitors. And that's the bit we've chosen to copy.

## Copyright and License

Copyright (c) 2020 James Laver, async-backplane Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

