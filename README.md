# Bevy Sodium

A conceptual port of the Sodium Functional Reactive Programming
library to Bevy.

## Licence

BSD 3-Clause, in [`LICENSE`](LICENSE) -- the same licence `sodium-rust` and the
other Sodium ports carry, so that a port is under the same terms as the thing it
ports.

One directory is **not** covered by it.
[`docs/reference/sodium/`](docs/reference/sodium/) holds material copied from
upstream Sodium, under its own BSD 3-clause licence (Copyright (c) 2015, Stephen
Blackheath) in
[`docs/reference/sodium/LICENSE`](docs/reference/sodium/LICENSE). The carve-out
lives here rather than as a preamble inside `LICENSE` so that the licence file
stays a byte-exact BSD 3-Clause: upstream Sodium's own root `COPYING` takes the
preamble approach, and GitHub reports it as no recognised licence.
