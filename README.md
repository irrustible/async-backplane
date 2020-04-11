# Tingle

I wanted to call it tangle but someone already took that crate name :/

Some experiments in erlang-style concurrency in rust.

## Status.

It doesn't work yet. It's just some stuff we'll need when it does work.

## Erlang/Rust comparison

| Feature         | Erlang    | Rust               |
|-----------------|-----------|--------------------|
| Shared mutation | No        | No                 |
| Concurrency     | Processes | Futures            |
| Messaging       | Mailboxes | Channels           |
| Backpressure    | No        | Optional           |
| Links           | Yes       | Kinda (JoinHandle) |
| Supervision     | Yes       | Not really         |
| Selective recv  | Yes       | Can be emulated    |

I think processes and futures are pretty comparable, but mailboxes and
channels are somewhat different in some aspects:

1. Erlang mailboxes have no backpressure.
2. Selective receive is arguably a workaround for lack of channels and
   has performance implications.
3. Mailboxes are single consumer, channels may be multi-consumer.
4. Mailboxes are tied to a process (biology model - send it a message).

This last point is interesting. a Pid serves a dual purpose in erlang,
in that as well as being a reference for process control
(i.e. termination), it is the handle to send it messages. This is a
convenience, but not an essential property - in fact we'd like to be
able to separate the two where it's convenient.

## So what will we build?

### Links

Links are the fundamental basis upon which supervision is built and
supervision is one of the most important features upon which erlang
builds its reliability.

The idea is simple: when a process terminates, you will be
notified. As the most common use case is to propagate failure, by
default a process will terminate with an error when a linked process
terminates with an error. It is also possible to override this and
perform custom logic, which is how erlang builds supervisors.

### Supervisors

Once we have links, building supervisors will allow us to build
supervision trees. And unlike bastion, these will be trees.

A supervisor spawns child tasks, listens for their exit and applies a
recovery strategy in response. This could range from doing nothing to
restarting all of the other children in the pool, to restarting itself
with a rate limit.
