# Tingle

A WIP minimalist asynchronous reliability backplane inspired by Erlang.

## Overview

Tingle is a minimalist asynchronous reliability backplane - a means of
structuring our async programs to achieve reliability. 

`Future` already gives us a simple structure for modelling potentially
incomplete asynchronous computation. Combined with an executor and
rust's async await syntax, we effectively get the green threads
experience at a much lower cost.

Tingle tries to bring Erlang's legendary reliability to rust in a way
that is easy and fun to use. Build highly reliable, efficient sytems!

## Status.

Getting close to alpha? Doesn't actually run yet but the design is
more or less there. Coming soon!

## Theory

Tingle is a 'reliability backplane' for `futures`-based
applications. Participating Futures connect to the backplane through a
`Device`. Devices may connect to other Devices through `Lines` to
notify each other of their disconnection from the backplane.

When connecting lines, there are two sides to consider:
* Devices whose disconnection I wish to know about.
* Devices which wish to know about my disconnection.

We say we 'monitor' the first kind and 'attach' the second kind. If we
both attach and monitor, we say we 'link'. A Device implements
`Stream`, so you can poll it for disconnections of monitored Devices.

With `Supervised`, you can wrap a Future such that it automatically
polls for disconnections when polling the inner future. If a linked
Device exits for any reason other than that it successfully completed,
`Supervised` will resolve to `Err(Crash)`. If the inner future exits
with `Ok(val)`, it will resolve to `Ok(val)`. If the inner future
exists with `Err(err)`, it will return `Err(Crash::Failure(err))`.

Next up, we will deal with Supervisors, which are special processes
designed to deal with failure.

## Copyright and License

Copyright (c) 2020 James Laver, tingle Contributors

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
