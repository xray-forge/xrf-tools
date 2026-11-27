# xrf-session

A value opened from game data as its readers hold it: an immutable snapshot under the identity of the opening that
produced it, with the parts read apart from the value kept beside it. It knows no application shell: a host decides how
a value is opened, when a snapshot is replaced by the next, and who is told.

## The snapshot

`SessionSnapshot<T>` is one opening's value at one revision of the mounted files ([xrf-vfs](../xrf-vfs/README.md)'s
`XrayRevision`):

- `session_id`: the `SessionId` of the opening, which every later snapshot of it keeps.
- `value`: the value, shared with the next snapshots while what it was read from stays current.
- `inputs`: what the open read to derive the value, an `XrayInputRecord`.
- `parts`: what readers derive from the value, a `SessionParts`.

`SessionSnapshot::new(session_id, value)` makes one with no inputs and no parts. `answer` and `map` wrap a value read
from a snapshot under its identity and revision, so whoever receives it can tell an answer older than what it holds.
`with_derived` makes the snapshot a host publishes when it derived the value again, from a `SessionDerived`.

## Parts

A part is a type implementing `SessionPart`: its `Value`, the `NAME` a change reports it by, and whether it is built
on a thread of its own (`IS_POOLED`, for a build that fans out over a pool whose workers may wait on it).

`SessionParts` keeps each part the first time anything asks and until the snapshot is let go:

- `get_or_init` builds a part that reads nothing; `get_or_record` builds one through a search recording what it reads.
- `record` and `record_read` record later reads into a part's inputs.
- `get` answers a part only where it was built.
- `with` keeps a part the opening already read.
- `to_current(vfs)` makes the store a next snapshot starts with: every built part whose inputs are current in `vfs`,
  and the names of those not carried.

## Features

- `typescript-bindings` derives the Specta types of `SessionId` and `SessionSnapshot` for generated mirrors.
- `fixtures` adds test helpers: the parts' recorded inputs. `SessionId::generate` is always there, for a native host
  allocating its own ids.

## Checks

Run `cargo test --locked -p xrf-session` from the repository root. The types are exercised by their host's tests, in
`xrf-app-core`.
