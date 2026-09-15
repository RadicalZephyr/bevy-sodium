# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Commands

Everything runs from the repository root. `--workspace` matters: the experiments
sub-project is a member, and leaving it out silently skips it.

| Task | Command |
| --- | --- |
| Build | `cargo build --workspace` |
| Lint | `cargo clippy --workspace --all-targets` |
| Format | `cargo fmt --all` (`--check` to verify) |
| Test | `cargo test --workspace` |
| Known gaps | `cargo test --workspace -- --include-ignored` |
| One test | `cargo test -p bevy-sodium NAME` -- add `-- --exact` for a whole path rather than a substring |
| Run an experiment | `cargo run -p adr-research --bin NNNN-record-name` |

`bevy` is depended on with `default-features = false`. It is a deliberate
choice, not an oversight: this crate models a dataflow graph in the ECS and
needs none of the renderer, windowing or asset stack, so it pays for none of
them. Reach for a feature when something here actually needs it, one at a time,
and expect the cold build to grow. The first build of the day compiles `bevy`
either way and is not quick; everything after it is incremental.

There is no binary and no example -- `bevy-sodium` is a library, so there is
nothing to `cargo run` in the root crate.

## Architecture

The premise, from [`README.md`](README.md), is a conceptual port of the
[Sodium](https://github.com/SodiumFRP) FRP library to Bevy. The bet that makes
it more than a translation is in [`src/lib.rs`](src/lib.rs): **the FRP
dependency graph is not a data structure the library owns, it is the ECS.** A
node is an entity, and an edge between nodes is a Bevy relationship. Sodium's
own implementations keep an explicit graph of nodes and hand-manage its
invariants; here the World is the graph, and the queries, change detection and
scheduling that Bevy already has are what walks it.

Read the crate with that in mind and everything currently in it is one edge
type:

- `DependsOn(Entity)` sits on the **dependent** node and names what it reads
  from. It is a Bevy `Relationship`, so it is the source of truth.
- `DependedOnBy(Vec<Entity>)` sits on the **depended-upon** node and collects
  every dependent. It is the `RelationshipTarget`, which means Bevy's component
  hooks insert, update and remove it automatically whenever a `DependsOn` moves.

**Never write `DependedOnBy` by hand.** Inserting `DependsOn` is the only
supported way to make an edge, and the target side is derived from it. Mutating
the derived side directly desynchronises the pair, which is why Bevy's derive
requires the collection field to stay private.

### The open question a reader will hit first

A Bevy `Relationship` is one-to-many: an entity may point at **at most one**
entity through a given relationship component. So a node carries one
`DependsOn`, and the graph as sketched expresses fan-out -- one cell feeding
many dependents -- but not fan-in.

Fan-in is not optional for this library. `lift`, `merge` and `snapshot` are all
nodes with two or more inputs, so the sketch does not yet reach Sodium's core
combinators. The ways out are known and none is obviously right: several
distinct relationship types (`DependsOnLeft` / `DependsOnRight`), an entity per
edge rather than per dependency, or a collection component that gives up the
relationship machinery and its hooks.

That choice is not made yet, and it is precisely the shape of decision this
repository records rather than leaves in the code. Anyone picking it is writing
`0002` before writing the type. The same goes for the rest of the port: Sodium's
semantics come from outside and are not ours to choose, but how they are spelled
against an ECS is entirely ours, and the spelling is where the reasoning lives.

Both types are currently private, so the crate exports nothing and `cargo build`
reports two `dead_code` warnings. That is the honest state of a sketch, not a
defect to paper over with `pub`.

## Conventions

### Conventions have an owner

Every convention in this repository is owned by one document, and the rest are
restatements. The ADR rules are owned by
[`docs/decisions/README.md`](docs/decisions/README.md); what appears here and in
[`CONTRIBUTING.md`](CONTRIBUTING.md) restates it. `CONTRIBUTING.md` owns the
rest: the checks a change has to pass and the rule about writing only the
source-of-truth half of a relationship pair, both of which the *Commands* and
*Architecture* sections above restate at an agent's level of detail. Change the
owner first, then every restatement. A restatement that has fallen behind is
worse than none, because it is read with the same confidence as the owner.

Anything normative -- a convention, a gate, a process, anything that changes
what someone does -- has to reach `CONTRIBUTING.md`. Write it there for a human
contributor and here for an agent where the register differs, but never
introduce a rule here that a contributor reading only `CONTRIBUTING.md` would
miss. A rule sequestered in an agent's briefing is a rule the people doing the
work never saw.

### Architecture decision records

Decision records live in [`docs/decisions/records/`](docs/decisions/records/),
one per file, `NNNN-kebab-case-title.md`. They are **living documents** --
edited to stay current rather than frozen on acceptance, because git is the log
and the document is the projection. New information goes in as a **dated
addition marked as arriving after the decision**, never as a silent revision of
the original reasoning. Read the directory as the current state of this
repository's decisions.
[`0001-recording-important-decisions.md`](docs/decisions/records/0001-recording-important-decisions.md)
argues for all of it; [`README.md`](docs/decisions/README.md) one level up has
the rules, and where a conflict or trade-off came up it belongs in the section
it concerns rather than a changelog at the bottom.

A record's status is a **dated transition log** in a collapsed `<details>` block
at the top of the file, with the current state in the `<summary>` line:

```markdown
<details>
<summary><strong>Status:</strong> Implemented 2026-11-03</summary>

| Date | Transition |
| --- | --- |
| 2026-09-10 | Drafted |
| 2026-09-24 | Accepted |
| 2026-11-03 | Implemented |

</details>
```

Transitions are `Drafted` (replacing a separate `Date` field), `Accepted` (a
commit before the record's PR merges), `Implemented` (the PR that finishes the
work), `Superseded by NNNN`, and `Deprecated`. The last row is the current state
and the summary restates it.

Two things the log is **not**. It is not an edit log -- only state transitions go
in it, and changes to a record's content are dated additions in the body next to
the reasoning they concern. It is not a changelog at the bottom -- it records
state, never reasoning, which is what keeps it consistent with writing conflicts
and trade-offs into the section they belong to. A row that wants a sentence of
explanation means that sentence belongs in the body.

Editing versus superseding: edit while the decision is still the decision; write
a new record when someone following the old one would now do the wrong thing.
Mechanically -- if the change can be a dated addition it is an edit; if it means
deleting a claim someone may have acted on, the old claim earns its own record.

**Any code that produces concrete data used in the argumentation of an ADR must
be committed somewhere a reader can run it** -- a number quoted in an ADR has to
be re-derivable from a checkout, or the record is asserting rather than arguing.
Which of two homes depends on what the experiment needs:

- **Needs this project's code** → the sub-project at
  [`docs/decisions/experiments/`](docs/decisions/experiments/), as an entry
  point named after the record (`0001-some-decision`), run with the command its
  README names. A dependency added there goes through the same checks as the
  rest of the repository; not publishing it is no reason to skip that.
- **Needs only the language's toolchain and standard library** → a share link
  on the playground named in `docs/decisions/README.md`, recorded in the ADR.
  This is also the only route for a build-time experiment: a case that must
  *fail* to compile or type-check cannot be an experiment entry point, because
  a sub-project that does not build breaks the project's build. If the README
  names no playground, everything that builds goes in `experiments/`, and a
  must-fail case is recorded in the ADR as the four parts below with the
  provenance line cut to `TOOL VERSION (released DATE)` and no link.

**Develop the experiment locally and mint the link last.** Run it with the
local toolchain against a file in your scratch directory until it produces the
result the record will quote, and only then create the share link. Never mint
one to find out what happens, and never rework an experiment after minting --
the superseded links stay live and nobody can collect them. **At most one share
link a minute**; if a record needs more than that, ask the user to mint them
rather than minting them faster.

A playground record is four parts in a fixed order: a bolded label saying what
it demonstrates, the source in a code block, its output in a `text` block, and a
provenance line as a blockquote beneath them --

```text
> TOOL VERSION (released DATE) - output checked DATE - [PLAYGROUND](URL)
```

Keep the fences bare; the blockquote is only the provenance line, which renders
muted. Both dates are load-bearing: the released date belongs to the toolchain
that produced the quoted output, the checked date is when the link was last
confirmed to still produce it. **The version stamp is required and is checked in
review** -- a link that names a channel rather than a version drifts, and
without the stamp a reader cannot tell whether the toolchain moved or the record
was wrong. `docs/decisions/README.md` has the skeleton.

An experiment is maintained while the decision it serves is still being argued
or built, and leaves the moment that decision is done -- which the record dates
exactly, in the `Implemented` row of its status log, or in the `Deprecated` row
if the decision was withdrawn rather than built. It goes by one of two exits --
**deleted** (it measured internals the ADR replaced; an entry point that no
longer builds is deleted, not repaired, and the ADR cites the commit that
produced its numbers) or **promoted** (it still answers a live question, so it
stopped being research: a measurement worth re-running moves to the benchmark
suite, a property we promise moves to the test suite). It never lingers.

So do not fix up an experiment that the project's checks break on without first
checking whether its record has logged `Implemented` or `Deprecated`; most
records argue for changing the internals the experiment was measuring, and
breaking is the expected end of its life. The sub-project is a staging area,
not an archive, and should trend toward empty.

### Research is evidence, not a test

**Do not write tests to motivate an ADR.** A test written against the structure
an ADR exists to replace has to be rewritten when the change lands: it guarded
nothing and only enlarged the diff. Support the argument with an experiment in
`docs/decisions/experiments/`, and let the existing suite keep checking that
behaviour did not change while the internals did.

**One exception, and it is the one this project hits most.** A record arguing
that the implementation diverges from *Sodium's semantics* is not arguing for a
design -- it is a bug report against a specification this project does not own.
That record does get a test, because the objection above does not apply: it is
written against the specification rather than against the internals the record
replaces, so it is already correct when the gap closes.

Put it in `src/tests.rs`, never in `docs/decisions/experiments/`, write it as
the behaviour the library *ought* to have so that it fails, and mark it
`#[ignore = "ADR-NNNN: one line"]`. The suite stays green, the reason prints on
every run -- the reason the attribute beats a quarantine module or a
`should_panic` -- and `cargo test --workspace -- --include-ignored` runs the
gaps. **Take the `#[ignore]` off in the pull request that closes the
gap**, the same one that logs `Implemented` -- that is what keeps the row
honest, so never log `Implemented` while the record's test is still ignored.

The test is only for divergence we mean to *close*. This is a conceptual port,
so a divergence we mean to keep is a decision and gets a record saying so
instead. Sorting the difference into one or the other comes before writing
either.
[`docs/decisions/README.md`](docs/decisions/README.md#the-one-exception-a-known-gap-against-sodiums-semantics)
owns the rule.

[`CONTRIBUTING.md`](CONTRIBUTING.md) states this for human contributors, and
carries process that never appears here. Read it before changing how anything
in this repository is done -- it is not a restatement of this file, and treating
it as one is how a convention ends up in only one of the two.
