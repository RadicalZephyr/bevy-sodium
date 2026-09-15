# Sodium reference material

Everything in this directory is **copied from upstream Sodium**, not written
here, and is covered by the BSD 3-clause licence in [`LICENSE`](LICENSE)
(Copyright (c) 2015, Stephen Blackheath) rather than by the repository's own
MIT-or-Apache-2.0 terms at the root. Both the licence and the copyright holder
differ, which is why this directory carries its own.

| File | What it is |
| --- | --- |
| [`denotational-semantics.md`](denotational-semantics.md) | The formal specification of Sodium's semantics, revision 1.1 |
| [`LICENSE`](LICENSE) | The licence that covers this directory, verbatim from upstream |

## Why it is in the tree

bevy-sodium is a port, and
[`docs/decisions/README.md`](../../decisions/README.md) turns on a distinction
that needs something concrete on one side of it: Sodium's semantics come from
outside and are not ours to choose, while how they are spelled against an ECS
is entirely ours. This document **is** that outside. It is what
"Sodium's semantics" refers to in a decision record, and it is what a known-gap
test is written against -- which is exactly why such a test stays correct after
the fix, where one written against our own internals would not.

Keeping it in the tree rather than as a link is deliberate. A specification a
reader has to go and find is one they will argue from memory instead, and a
link is a promise about someone else's hosting. A test that cites `5.4` should
be one click from `5.4`.

It is a **reference**, which is a third thing beside the two the decisions
directory knows about. It is not a decision, so it has no status log and no
transitions. It is not an experiment, so it has no scheduled exit and does not
belong in `experiments/`. It changes only when upstream changes.

## Provenance, and what was changed

The specification is published upstream at
[`SodiumFRP/sodium/denotational/`](https://github.com/SodiumFRP/sodium/tree/master/denotational)
as an `.odt` and a `.pdf`, beside `sodium.hs`, the executable version the
document refers to. That directory carries its **own** `LICENSE`, which is the
one copied here, distinct from the licence at the root of that repository.

The Markdown transcription came by way of the same specification as reproduced
in *Functional Reactive Programming* (Blackheath & Jones) as its Appendix E,
which is the version that had already been converted to Markdown. Its content
is the revision 1.1 document -- same revision history, same sixteen primitives,
same figures -- but the transcription carried the book's apparatus, and that
apparatus is the publisher's rather than part of the licensed document. It was
stripped:

- **Ebook index anchors** (`<a id="iddle1181"></a>` and some two hundred more)
  and section-id anchors (`<a id="app05lev1sec1"></a>`) removed. They pointed
  into a book index that does not exist here.
- **The appendix letter dropped** from section and figure numbers, so `E.5.4`
  is `5.4` and `Figure E.4` is `Figure 4`. The mapping is exact in both
  directions, so a citation of either form still finds the right section.
  Headings moved up one level to match, the title now being the document's own
  rather than an appendix of something else.
- **Figure anchors renamed** from `app05fig04` to `fig-4`, and the twenty
  cross-references updated to match. All twenty resolve.

The text, the Haskell, the semantic definitions and the diagrams are untouched.
Blocks fenced ` ```swirly ` are the timing diagrams as the source wrote them;
they read as plain text without any tool.

To check any of this against the original, the `.pdf` upstream is the same
revision.
