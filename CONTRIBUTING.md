# Contributing

## Architecture decision records

Decisions whose *reasoning* is the valuable part get a record in
[`docs/decisions/records/`](docs/decisions/records/). Records are living
documents -- edited to stay current, with new information added as a dated note
marked as arriving after the decision, never as a silent revision of the
original reasoning. Read the directory as the current state of the project's
decisions, and reach for `git log -p` when you want to know how it got there.
[`docs/decisions/README.md`](docs/decisions/README.md) has the conventions;
[`0001-recording-important-decisions.md`](docs/decisions/records/0001-recording-important-decisions.md)
argues for them.

A record's status is a dated transition log in a collapsed block at the top of
the file, and each row is a transition you log deliberately:

| When | Row you add |
| --- | --- |
| writing the record | `Drafted` |
| a commit before its pull request merges | `Accepted` |
| the pull request that finishes the work | `Implemented` |
| a commit before a superseding record merges | `Superseded by NNNN` |
| when a reversal is written into the record | `Deprecated` |

`Superseded` points outward, to the record that replaced this one.
`Deprecated` points inward: the summary links to the section of this record
explaining the reversal, which is why withdrawing a decision needs no successor
record to stay accountable. A decision abandoned before anything was built on
it is usually `Deprecated`; one that shipped and was later replaced is
`Superseded`.

The current state is the last row, restated in the summary line so a reader who
never expands the block still knows where the record stands. The gap between
`Accepted` and `Implemented` is the number worth watching -- a decision the code
never honoured is a row that never arrived.

Only state transitions belong in the log. A change to what a record *says* is a
dated note in the body, beside the reasoning it concerns.

Code that produces concrete data used in the argumentation of an ADR has to be
committed somewhere a reader can run it -- a number quoted in an ADR has to be
re-derivable from a checkout, or the record is asserting rather than arguing.
An experiment needing this project's code goes in the sub-project at
[`docs/decisions/experiments/`](docs/decisions/experiments/) as an entry point
named after the record; one needing only the language's toolchain and standard
library goes in a playground share link recorded in the ADR.
[`docs/decisions/README.md`](docs/decisions/README.md#what-the-playground-has-to-be)
names the playground and says what one has to provide. Where it names none,
everything that builds goes in the sub-project, and a case that has to *fail*
to build is recorded in the ADR with no link.

The playground is for recording a result, not for finding one. A share link is
minted on someone else's infrastructure, and it publishes a snippet that nobody
can collect afterwards. So an experiment is developed **locally** -- the
toolchain against a file in a scratch directory, as many times as it takes --
and goes to the playground only once it produces the result the record is going
to quote. Nothing is minted to find out what happens; a link is minted to
publish what already happened.

Two consequences, stated outright because the natural working rhythm violates
both. **No iterative development against playground resources**: an experiment
that took four attempts should leave one link behind, not four, and reworking
after minting turns the earlier links into litter that stays live and wrong. And
**at most one link a minute** -- a record needing several is a record that should
mint them at that pace.

A playground experiment is written as four parts in a fixed order: a bolded
label saying what it demonstrates, the source in a code block, its output in a
`text` block, and a provenance line as a blockquote beneath them, reading
`TOOL VERSION (released DATE) - output checked DATE - [PLAYGROUND](URL)`.
The fences stay bare. Both dates matter: the released date belongs to the
toolchain that produced the quoted output, the checked date is when someone last
confirmed the link still produces it.
[`docs/decisions/README.md`](docs/decisions/README.md#playground-experiments-carry-four-things-not-one)
has the skeleton.

**Reviewing a record includes checking two things.** First the version stamp: a
playground link re-runs against whatever its channel points at when it is
clicked, so any toolchain output quoted in a record has to say which version
produced it, and a record carrying none cannot be checked later. Second the
status log, which needs its `Accepted` row before the record merges. Neither
should be approved on autopilot.

## Tests and research are not the same thing

An ADR arguing for a different internal design should not be accompanied by
tests. Such a test is written against the very structure the ADR exists to
replace, so landing the change means rewriting it -- it never guarded
anything, it just made the diff bigger. Support that argument with an
experiment in [`docs/decisions/experiments/`](docs/decisions/experiments/)
instead, and let the existing suite go on checking that behaviour did not
change while the internals did.

Research is evidence, not a test. It is not expected to keep passing, and it
has a scheduled exit:
[`docs/decisions/experiments/README.md`](docs/decisions/experiments/README.md#retirement)
says when and where it goes.
