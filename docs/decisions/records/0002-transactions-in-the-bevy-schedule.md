# 0002 -- Transactions in the Bevy schedule

<details>
<summary><strong>Status:</strong> Accepted 2026-09-15</summary>

| Date | Transition |
| --- | --- |
| 2026-09-15 | Drafted |
| 2026-09-15 | Accepted |

</details>

## Context

Every guarantee Sodium makes comes from the transaction. Glitch-freedom,
simultaneity, the atomicity of a cell's update -- none of them are properties of
`Stream` or `Cell` individually, they are properties of the instant those types
are evaluated at. So before any combinator can be spelled against an ECS, two
questions have to be answered: **what is an instant, and where does one
happen?**

The specification answers the first for Sodium in general.
[`denotational-semantics.md`](../../reference/sodium/denotational-semantics.md)
defines a type `T` that is a total order, extended hierarchically so that for
any time `t` we can add children numbered with natural numbers, all greater
than `t` but smaller than any greater sibling of `t` (section 4). It does not
answer the second, because the second is a question about the host, and the
host is ours to choose.

Bevy does not answer it either. **`bevy_ecs` 0.19.1 contains no transactional
primitive at all** -- no atomic multi-step world mutation, no staging area, no
deferred-then-committed state. A search of the crate for `transaction`,
`staging`, `two-phase`, `rollback` and `commit` returns one doc comment, on
`set_last_changed`, about rollback networking. So the transaction is not adopted
from Bevy; it is constructed out of what the scheduler does offer.

## Decision

1. **Time is hierarchical.** `T` is the spec's `T`, ordered as the spec orders
   it. A transaction is opened per *send*, never per frame.
2. **Sends queue; a runner drains them.** A send from a system enqueues rather
   than evaluating. One runner drains the queue, opening a transaction per
   queued send and giving each its own `T`.
3. **The runner is an exclusive system in `PreUpdate`.**
4. **`Messages` is transport, not boundary.** It may carry a send from a system
   into the queue. It does not define the instant.
5. **Names come from the specification.** `Stream`, `Cell`, `Transaction`,
   `send`. Where one collides with Bevy vocabulary it is module-qualified, not
   renamed: the spec is in this tree and a reader should be able to move between
   the two without translating.

## Why hierarchical time rather than a frame tick

The tempting simplification is one transaction per frame, with `T` as the frame
number. Bevy already batches that way -- `Messages` collects for a frame and
swaps once -- so the machinery would line up almost for free.

It is wrong, and wrong in kind rather than in precision. Simultaneity in Sodium
means *caused by the same external event*, not *arriving in the same window*.
`Merge` coalesces simultaneous events through its combining function (section
5.4), so two causally unrelated sends that happened to land in the same frame
would be combined into a single event. That is not a coarser answer than the
right one; it is a different and observably incorrect one, and every combinator
downstream inherits it.

The hierarchical form also turns out to have a native shape in Bevy, which is
weak evidence for it but worth recording. `Split` puts its values into newly
created child time steps `t++[n]` (section 5.10), strictly greater than `t` and
strictly less than `t`'s next sibling. Bevy's command queue drains with the same
ordering: commands queued *by* command N are applied after N and before N+1,
because each command's application is followed by a `world.flush()` that
re-enters the drain against a fixed snapshot. Depth-first across generations, in
both cases. Whether that correspondence can carry any weight is for the
implementation to find out.

## Why one exclusive system, and not one per send

The only construct in `bevy_ecs` 0.19.1 with a source-level guarantee that no
other system observes intermediate state is an **exclusive system**. The
multi-threaded executor will not spawn any task while one is running, and will
not start one until the count of in-flight systems reaches zero. It is a full
barrier in both directions. Every `ApplyDeferred` sync point is an exclusive
system too, so the schedule already pays this cost several times a frame.

Read naively, a per-send transaction means a barrier per send, which is the cost
that made a frame tick attractive in the first place. Queuing dissolves that:
the runner holds `&mut World` once and drains N sends inside it, so N
transactions
cost one barrier. The decision to queue is therefore not an implementation
convenience -- it is what makes per-send transactions affordable at all.

**The command queue is not the substrate.** It is the obvious candidate and it
cannot work, for precisely the property we are trying to provide: because
`world.flush()` runs after each individual command, a command queued during a
drain observes partially-applied state. Glitch-freedom is the rule that forbids
exactly that.

## Why `PreUpdate`

`FixedMain` is the natural home for game logic and it runs zero or more times
per frame, driven by an accumulator -- zero whenever less than one timestep has
accumulated. A reactive system that sometimes does not run at all in a frame is
worse than one that runs at a coarse granularity, because the failure is
intermittent rather than uniform.

`First` already hosts the `Messages` buffer swap, which is itself an exclusive
system and is registered `ambiguous_with` the time system, so their relative
order is deliberately unspecified. Adding a second exclusive system to that
neighbourhood puts the transaction runner in the one place where ordering is
explicitly disclaimed.

`PreUpdate` runs every frame, after the swap, before game logic. It is the
earliest point that is both unconditional and unambiguous.

## Consequences

**A whole-schedule barrier every frame, of unmeasured cost.** Nothing in
`bevy_ecs` quantifies the runtime cost of a sync point -- there is no benchmark
comment and no note calling `ApplyDeferred` expensive. We are adding one more
and will have to measure it ourselves.

**A one-frame latency floor.** A send made during `Update` is not evaluated
until the next frame's `PreUpdate`. For a library whose selling point is
coherent event handling, this is the most likely thing to make it feel wrong to
use, and it should be the first thing the experiments in `0003` look at.

**No abort.** `bevy_ecs` has no rollback; a panic part-way through a command
drain leaves already-applied commands applied. A transaction that fails
part-way has no defined recovery, and we are not in a position to give it one.

**Nothing warns when a system races the runner.** Bevy's schedule ambiguity
detection defaults to `LogLevel::Ignore`, so a system with no ordering against
the runner that conflicts on its data is silently permitted. Turning detection
on is one line of `ScheduleBuildSettings` and should probably be on in this
repository's own tests.

## What this record does not decide

**Whether FRP earns its place in Bevy at all.** That is a separate question with
a separate record. `0003` will argue it from a paired build -- the same feature
implemented twice, once in idiomatic Bevy and once with `sodium-rust` -- and is
about code shape only, with performance deliberately out of scope.

**Fan-in.** A Bevy relationship is one-to-many, so a node names at most one
dependency and the sketch in `src/lib.rs` reaches fan-out but not the fan-in
that `lift`, `merge` and `snapshot` require. Still open.

**Driving the schedule from the dataflow graph.** Bevy supports injecting
computed dependency edges at build time through the `ScheduleBuildPass` trait,
and `AutoInsertApplyDeferredPass` is a working example of one in the engine
itself. Using the FRP graph's topology as scheduler input is therefore possible
rather than hypothetical. It is recorded here so it is not rediscovered, and
deliberately not taken: a schedule rebuild recomputes the executor's conflict
matrix over every pair of systems, and none of it is worth designing around
before the graph works.

## Alternatives considered

**One transaction per frame, `T` = frame number.** Rejected above: it coalesces
causally unrelated sends through `Merge`'s combining function, which is a
semantic error rather than a loss of resolution.

**One transaction per fixed timestep.** Same defect, and `FixedMain` can run
zero times in a frame.

**Building the transaction on the command queue.** Rejected: nested commands
observe partially-applied state.

**The runner in `First` or in `FixedMain`.** Rejected above.

**Bridging `sodium-rust` into Bevy instead of porting.** This would inherit
every semantic guarantee already implemented and tested, for a fraction of the
work, at the cost of a single-threaded graph behind a lock and an FRP graph the
ECS cannot see. It is turned down by intention rather than by argument: this
repository exists to find out what an ECS-native spelling looks like, and a
bridge answers a different question. It remains the fallback if the port proves
unusable, unidiomatic or ruinously slow.

## Evidence

Everything claimed here about Bevy was read from the vendored sources of
`bevy_ecs` 0.19.1, `bevy_app` 0.19.1 and `bevy_time` 0.19.1 on **2026-09-15**.
These are third-party capabilities and will age; a later reader disagreeing with
a claim should check the version first.

No experiment accompanies this record. Its claims are citations to another
project's source and doc comments rather than measurements, and the routing rule
in [`README.md`](../README.md) applies to code that produces data. The two
claims most likely to need measuring later -- the cost of the added barrier and
the one-frame latency floor -- are consequences to be tested once something
runs, not premises this decision rests on.

No test accompanies it either. This is a decision about how Sodium's semantics
are *spelled* against an ECS, not a claim that an implementation diverges from
them, so the exception in
[`0001`](0001-recording-important-decisions.md#except-where-the-subject-came-from-outside)
does not apply.
