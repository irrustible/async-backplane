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

Getting close to alpha? Doesn't actually run yet.

<!-- ## Usage -->

<!-- Don't. But here's what it might look like when it works: -->

<!-- ```rust -->
<!-- use core::time::Duration; -->
<!-- use tingle::{Entanglement, Kind, Quantum}; -->
<!-- use piper::chan; -->

<!-- async fn root_supervisor(&mut q: Quantum) { -->
<!--   q.spawn(app_supervisor, Kind::Supervisor); -->
<!-- } -->

<!-- async fn app_supervisor(&mut q: Quantum) { -->
<!--   q.spawn_link(Kind::Supervisor); -->
<!-- } -->

<!-- fn main() { -->
<!--   tingle::run(root_supervisor) -->
<!-- } -->
<!-- ``` -->

<!-- ## Guide -->

<!-- Participating `Future`s ("quanta") are spawned onto an executor with a -->
<!-- `Quantum`, a handle into the backplane. Quanta may observe the -->
<!-- termination ("decoherence") of other quanta through a process known as -->
<!-- "entanglement". -->


<!-- It may observe the `Decoherence` of other Quanta when -->
<!-- they finish executing -->

<!-- A `Quantum` corresponds to an erlang process - a logical concurrent -->
<!-- thread of execution. It may `entangle` and `untangle` (undo -->
<!-- entanglement) with other quanta if it has their addresses to do -->
<!-- so. When the `Quantum` exits, it will notify all entangled quannta. -->

<!-- Under the hood, the set of interactions between two quanta are limited: -->

<!-- * request to entangle with the other -->
<!-- * request to untangle (undo entanglement) from the other -->
<!-- * request the other to exit -->
<!-- * notify the other of our exit -->

<!-- Processes may respond to exit notifications differently. The default -->
<!-- behaviour is to exit if the result is considered a failure (as -->
<!-- modelled by the simple `Superposition` trait, which is already -->
<!-- implemented for `Result`). -->

<!-- A `Supervisor` disables the default behaviour and applies a recovery -->
<!-- strategy, which may involve restarting other processes or in grave -->
<!-- circumstances, exiting itself (delegating to *its* supervisor the -->
<!-- responsibility for restarting it). -->

## Naming

The QM analogy was inspired by the idea of two processes having their
fate linked by entanglement. Erlang's uses the term 'links' (although
'monitors' are related to `Observer` entanglement). It's also a quiet
tribute to Joe Armstrong (the inventor of Erlang), who was a physicist
before he was a programmer.

Yes, we've taken a little bit of artistic license in interpretation,
but there's a pleasing similarity with some aspects of QM.

I wanted to call this library 'Tangle' but someone already took the
crate name.

## Copyright and License

Copyright (c) 2020 James Laver.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
