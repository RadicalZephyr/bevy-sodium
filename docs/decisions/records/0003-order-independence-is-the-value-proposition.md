# 0003 -- Order independence is the value proposition

<details>
<summary><strong>Status:</strong> Drafted 2026-09-15</summary>

| Date | Transition |
| --- | --- |
| 2026-09-15 | Drafted |

</details>

## Context

This library started from a supposition: Bevy's event handling is the observer
pattern, the observer pattern has the six plagues that *Functional Reactive
Programming* (Blackheath & Jones, 2015) was written to banish, therefore
porting Sodium to Bevy would fix them. Every
step of that was assumed. Before spending real effort on
[`0002`](0002-transactions-in-the-bevy-schedule.md)'s transaction, the
supposition needed testing.

Two things sharpened the question before any code was written.

The first is that "Bevy's events" is not one thing. In 0.19.1 `Message` is the
buffered, per-cycle mechanism and has no listeners at all; `Event` is the
registered-callback one and is not buffered. They have close to opposite
plague profiles, and a claim about "Bevy's event system" is not a claim about
either.

The second is that the plague list is not the interesting question. If the
library's pitch is "fixes six problems from a 2015 book about GUI listeners",
its scope is set by that book. What is actually missing from Bevy is narrower
and more defensible, and this record names it.

## Decision

**The library is worth building, and its value proposition is order
independence.**

Bevy's documentation says the relative order of observers watching the same
event "is considered to be arbitrary" and recommends making no assumptions
about it. That is a request, not a guarantee. Nothing checks that your result
is actually independent of the order, and ambiguity detection defaults to
`LogLevel::Ignore`, so nothing warns you either.

FRP makes the same order undetectable *because the semantics guarantee the
result does not depend on it*. Those two positions sound alike and are not the
same thing at all. The gap between them is what this library sells.

**The six plagues are the motivation, not the claim.** They are why Blackheath
and Jones went looking for FRP; they are not a specification of what FRP is
worth, and scoping to them would import someone else's problem list.

## The evidence: one slice, built twice

The slice is deliberately the smallest thing with a **diamond** -- a value
derived from two inputs that can both change in the same instant. A player has
health, a maximum that grows on level-up, and a shield that absorbs damage
before health sees it. Two values are derived: the health fraction a bar would
show, and effective HP.

The instant applies all three inputs at once: heal 100, level up +100 max,
damage 50. The heal clamps to the maximum, so the heal and the level-up meet at
one node, and the damage and the shield meet at another.

It was built in two stages on purpose. Stage 1 has no shields; stage 2 adds
them, which is the requirement change -- a new input feeding an existing node,
plus a second derived value. The cost of stage 2 is the measurement.

### Arm A: Bevy observers

[`0003-health-shield-bevy.rs`](../experiments/src/bin/0003-health-shield-bevy.rs),
run with `cargo run -p adr-research --bin 0003-health-shield-bevy`:

```text
registered heal first  -> health  80 / max 200 / shield  0
                          derived fired 3x  fraction [1.0, 0.8, 0.4]  effective [130, 80, 80]
registered damage first -> health 100 / max 200 / shield  0
                          derived fired 3x  fraction [0.4, 1.0, 0.5]  effective [40, 100, 100]
derive afterwards      -> health  80 / max 200 / shield  0
                          derived fired 1x  fraction [0.4]  effective [80]
```

> bevy 0.19.1 - output checked 2026-09-15 - stage 2 at commit `b4d3097`

Three things in that output.

**The final state depends on observer order.** Health is 80 or 100 for the same
instant. Not a transient that settles -- a different permanent value, chosen by
an order Bevy declines to define.

**The derived values fire once per mutating observer**, and the intermediate
values are not merely early, they are wrong: `1.0` is a full health bar at a
moment the player is being hit, `0.4` in the other ordering is computed against
a maximum that is about to double. Anything reacting to these sees states that
never existed.

**The cost of the requirement change was multiplicative.** Stage 2 added one
observer and one derived value, and took the number of recompute sites from two
to six: every observer that mutates an input has to recompute every derived
value, because nothing in Bevy knows they are derived.

### The steelman, and what it does not fix

The third row is the fix a Bevy reviewer would reach for, and it is included
because a record that does not steelman the incumbent is a sales pitch.
Observers only mutate; the derived values are computed once, afterwards, the way
a later system with change detection would.

It works. One firing, no bogus intermediates. **And the final state is still 80
rather than 100** -- still whichever the observer order produced. The obvious
fix addresses the symptom and leaves the cause untouched, which is the clearest
single result in this record.

### Arm B: incomplete, and why

[`0003-health-shield-frp.rs`](../experiments/src/bin/0003-health-shield-frp.rs)
builds the same slice with `sodium-rust` 2.1.3. Its graph is the artifact worth
comparing -- inputs merge into one stream of deltas, the two derived values are
each declared once as functions of their inputs, and the requirement change
wires a new input in at one place rather than at every mutation site.

**Its numbers are not trustworthy, because building it surfaced a correctness
bug in `sodium-rust`.** Adding `health.lift2(&shield, ..)` -- a derived cell
nothing else reads -- stops `health` updating at all.

[`0003-lift2-loop-bug.rs`](../experiments/src/bin/0003-lift2-loop-bug.rs) is the
minimal reproduction, `cargo run -p adr-research --bin 0003-lift2-loop-bug`:

```text
no lift2                     health = 100  ok
lift2(max_health, health)    health = 100  ok
lift2(health, shield)        health =  60  WRONG
both                         health =  60  WRONG
```

> sodium-rust 2.1.3 - output checked 2026-09-15 - commit `b4d3097`

`lift2` is `Apply` in the specification, section 5.14: a pure function of two
cells, which cannot change either of them. The shape that triggers it is a
diamond through a loop -- `health` is held by a `CellLoop` and its own update
stream reads `shield`, so lifting `health` together with `shield` lifts a cell
with something upstream of itself.

That shape is not exotic. It is what "shields absorb damage before health" looks
like, and something very close to it will appear in any dataflow graph with
feedback. **The reference implementation is unreliable in exactly the shape this
library needs**, which is a finding about the port rather than an obstacle to
it, and is recorded here rather than routed around.

## The objection, and the answer

The fair objection to arm A is that nobody should put three observers that
mutate shared state on one event: use a single observer, or order them.

Ordering them is not available -- Bevy has an open issue for exactly that and
the docs say the order is arbitrary in the meantime. So the objection reduces to
"do not get into that situation", which is a real answer and also the point. The
event system imposes a constraint on the design rather than the design choosing
it, and the constraint is invisible: nothing in the type system, the API or the
default build settings tells you that you have three observers on one event
whose order decides your state.

A second objection is that arm A keeps its state in one `Resource`, so all three
observers conflict. Splitting them into separate resources or components changes
the glitch story but not the result: the order-dependence is in the sequence of
mutations, not in where they are stored.

## Consequences

**Development continues on `0002`'s terms.** The supposition survived contact,
in a narrower and better-evidenced form than it started with.

**The README's claim should be order independence**, not plague-banishing, and
not "FRP for Bevy". The narrowest true version of the pitch is the strongest.

**Correctness of the Rust FRP implementation is now a known risk**, not an
assumption. This cuts both ways for the port-versus-bridge decision recorded in
[`0002`](0002-transactions-in-the-bevy-schedule.md): a bridge would have
inherited this bug, and a port has to avoid writing its own version of it. It is
the strongest argument yet for a test suite written against the specification
rather than against the implementation.

**The lift2 bug should be reported upstream.** The reproduction is written to be
liftable into `sodium-rust`'s own test suite unchanged.

## What this record does not decide

**That FRP delivers order independence in practice.** This record establishes
that Bevy has the defect and that FRP's semantics answer it. It does *not* show
a working implementation producing the specified answer for this slice, because
the one available does not. That demonstration needs a third arm built on
`bevy-sodium`, and it is what `Implemented` will mean here.

**Anything about performance.** Deliberately out of scope: `sodium-rust` is
known to be slow, arm B is not ECS-native, and a code-shape comparison is not a
benchmark.

**Fan-in.** Still open, still blocking implementation.

## Experiment retirement

The two arms are maintained until `bevy-sodium` can run the same slice, at
which point they are either promoted -- the scenario becomes a test written
against the specification -- or deleted. `0003-lift2-loop-bug` leaves sooner:
it belongs in `sodium-rust`'s test suite, and is deleted from here once it
lands there or once the bug is fixed.

## Alternatives considered

**Auditing Bevy against the six plagues.** The original plan. Rejected once it
became clear the audit could not fail -- Bevy's own docs concede arbitrary
ordering and undetected propagation cycles in writing, so the exercise would
have produced citations rather than a decision. Those citations survive as
context above.

**A kata as the test case.** Bowling was considered and rejected: it is a fold
over a sequence, with no node depending on two sources, so nothing FRP offers
would have been exercised. Gilded Rose was kept as a possible second case -- its
Conjured-items twist is a clean built-in requirements change -- but it has the
same weakness, every item updating independently.

**Waiting for `sodium-rust` to be fixed before concluding.** Rejected: arm A
alone establishes the defect, and the bug is itself evidence worth recording now
rather than after it disappears from history.
