# Experiments

Experiments backing the decision records in
[`docs/decisions/records/`](../records/).

Any code that produces concrete data used to argue an ADR lives here rather
than in a scratch file or a gist. An ADR that cites a measurement is only as
good as a reader's ability to re-run it, and a benchmark that lived in
someone's working tree cannot be re-run at all.

## Layout

This directory is a sub-project of the main build. It depends on the project by
path rather than on a published version, is never published itself, and is
covered by the same checks -- lint, type check, tests -- as anything else in the
repository, because an experiment is still code someone runs, and not publishing
it is no reason to skip that. How it joins the build, and what it is called, the
project says here:

> **Sub-project:** a Cargo workspace member, the crate `adr-research`. Its
> shared-fixture module is `src/lib.rs`, which is empty and is also what gives
> the crate a target while no experiment is checked in.

Each experiment is one entry point named after the ADR it serves:

```text
docs/decisions/experiments/src/bin/0001-some-decision.rs
```

and is run from the repository root with one command:

```shell
cargo run -p adr-research --bin 0001-some-decision
```

The record's name is the only thing that varies from one experiment to the
next. Print results in whatever shape the ADR needs to quote them, and link the
entry point from the section that relies on the numbers. Fixtures shared between
experiments -- a data builder two of them both need, a timing harness -- get a
shared module, and it stays empty until something is actually shared. Resist
putting an experiment's own scaffolding there.

## What does not go here

Research produces *evidence*, not tests. It is not asserting that the project
is correct, and it is not expected to keep passing -- it answers a question
that was open at the time an ADR was written.

An experiment that needs nothing from this project does not belong here either.
If it depends only on the language's toolchain and standard library -- a probe
into type inference, a diagnostic worth quoting -- it goes in a playground share
link recorded in the ADR instead, which is also the only route available to a
case that has to *fail* to build. That holds while
[`../README.md`](../README.md) names a playground; where it names none, a
toolchain-only experiment that builds lives here after all, and a case that has
to *fail* to build is recorded in the ADR with no link. The same file has the
routing rule, what the playground has to provide, and the version stamp such a
record has to carry.

## Retirement

An experiment is maintained while the decision it serves is still being argued
or built. It leaves the moment that decision is done -- which the record dates
exactly, in the `Implemented` row of its status log, or in the `Deprecated` row
if the decision was withdrawn instead of built -- through one of two exits:

- **Deleted** -- it measured internals the ADR replaced. An entry point that
  no longer builds against the new code is deleted rather than repaired; cite
  the commit that produced the numbers in the ADR and history keeps it. Most
  records here argue for changing the internals an experiment was measuring, so
  breaking is how it ends rather than a regression.
- **Promoted** -- it still answers a live question, which means it stopped
  being research. A measurement worth re-running is a benchmark and moves to
  the benchmark suite; something asserting a property we promise is a test and
  moves to the test suite. A project with no benchmark suite starts one the
  first time this happens; the experiment does not stay here as the substitute.

What an experiment never does is linger. This sub-project is a **staging area,
not an archive**: everything in it has a scheduled exit, and a healthy one
trends toward empty. Accumulation is the signal to look for something
miscategorised -- a benchmark that was never promoted, or an experiment whose
ADR quietly landed months ago.
