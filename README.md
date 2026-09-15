# Bevy Sodium

A conceptual port of the [Sodium](https://github.com/SodiumFRP) Functional
Reactive Programming library to Bevy.

## Status

A sketch, and this README would rather say so than imply otherwise. The crate
defines one relationship pair, exports nothing, and has never been published.
What exists so far is the groundwork: the idea below, an in-tree copy of the
semantics being ported, and a convention for recording the decisions that are
about to be made.

## The idea

Sodium's implementations keep an explicit graph of nodes and hand-manage its
invariants. Here the graph is not a structure the library owns -- **it is the
ECS**. A node is an entity, an edge between nodes is a Bevy relationship, and
the queries, change detection and scheduling Bevy already has are what walk it.

That is the bet the port exists to test. The first thing it runs into is that a
Bevy relationship is one-to-many, so a node names at most one dependency: the
graph reaches fan-out but not the fan-in that `lift`, `merge` and `snapshot`
all need. Resolving that is the next decision, not a settled matter.

## Two halves

Sodium's semantics are not this project's to choose, so the specification is in
the tree rather than linked:
[`docs/reference/sodium/denotational-semantics.md`](docs/reference/sodium/denotational-semantics.md)
-- sixteen primitives, their definitions in Haskell, and a worked timing diagram
for each.

How those semantics are spelled against an ECS *is* this project's to choose,
and that half is what [`docs/decisions/`](docs/decisions/) records: what was
chosen, what was turned down, and why. Read that directory as the current state
of the project's decisions.

## Building

Everything runs from the repository root, and every command takes `--workspace`
so the experiments sub-project is not silently skipped.

```shell
cargo build --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
cargo test --workspace
```

`bevy` is depended on with `default-features = false`: this crate models a
dataflow graph and needs none of the renderer, windowing or asset stack.

## Contributing

[`CONTRIBUTING.md`](CONTRIBUTING.md) has the conventions, which are more
specific than most projects' -- how decisions are recorded, where evidence for
them lives, and the one narrow case in which a decision brings a test.

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. These are the terms Bevy itself carries, chosen so that nothing about
this repository's licensing blocks code moving upstream into Bevy later --
relicensing is free with one contributor and expensive with twenty.

One directory is **not** covered by them.
[`docs/reference/sodium/`](docs/reference/sodium/) holds material copied from
upstream Sodium under its own BSD 3-clause licence (Copyright (c) 2015, Stephen
Blackheath), in
[`docs/reference/sodium/LICENSE`](docs/reference/sodium/LICENSE). The carve-out
is stated here rather than inside the licence files so those stay byte-exact and
machine-detectable.
