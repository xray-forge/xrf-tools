# xrf-vfs

Indexes X-Ray assets and layers their physical sources in a virtual file system.

The engine never opens a game file by its host path. It asks for an engine path such as `configs\system.ltx`, and its
file system answers from whichever mounted source holds it first: a loose `gamedata` tree, an archive volume set, or a
mod tree in front of both. This crate reproduces that lookup. Plan a search over the places to read, open it, and read
through it. Installation plans read the mount order from `fsgame.ltx`, so an override resolves to the file the engine
would load. Format crates such as `xrf-ltx` and `xrf-gamedata` read through a search; the binaries own mounting.

## Quickstart

```rust,no_run
use xrf_vfs::{XrayMountMode, XraySearch, XraySearchPlan};

# fn main() -> xrf_error::XrfResult {
let search: XraySearch = XraySearchPlan::new()
  .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Auto)?
  .open()?;
let bytes: Vec<u8> = search.read("configs\\system.ltx")?;
# Ok(())
# }
```

## Concepts

- **Logical path** — an engine identity: lower case, `\`-separated, with no empty, `.` or `..` component and no colon.
  `XrayLogicalPath` is the typed form; every lookup accepts a raw `&str` and normalizes it, so `Configs/System.LTX`
  names the same asset.
- **VFS snapshot** — `XrayVfs` holds mounted sources (a loose directory or a `.db` volume set at a logical base) in
  priority order, the revisions of what they hold, and what it retains after parsing. It is read only through a search,
  which shares it behind an `Arc`, so a snapshot in use does not change: `commit` and `apply` answer the next snapshot,
  and a host mounts into a fork (`XrayVfs::fork`, which shares sources and cache).
- **Search** — `XraySearch` is an ordered list of steps over one snapshot it owns, first hit winning: what PhysicsFS and
  Source's file system call a search path. Each step is a label and an `XrayLookupScope` (which mounts, which logical
  subtree). A search has no lifetime, is cheap to clone, and can record every input it reads.
- **Mode** — `XrayMountMode` says how a caller's path becomes mounts. `Auto` (the default) reads the path as an
  installation when it holds an fsgame file, as one volume when it is one, as a volume set when it holds volumes
  directly, and as a loose root otherwise; it never searches upward. `Directory` forces a loose root, `Volumes` mounts
  every `.db`/`.xdb` volume beneath the path recursively, `Installation` requires an fsgame file, and
  `ContainingInstallation` searches upward for one, falling back to a loose root when none exists.
- **fsgame file** — the layout file that declares an installation, found by `FsgameFile::find`. Every OpenXRay-family
  engine reads `fsgame.ltx`; each Enhanced Edition's engine reads only its own game's name instead (`fsgame_soc.ltx`,
  `fsgame_cs.ltx`, `fsgame_cop.ltx`) and supplies `$fs_root$` and `$app_data_root$` itself. A directory is searched in
  `FsgameFile::FILE_NAMES` order, `fsgame.ltx` first. This crate anchors `$fs_root$` at the file's directory and leaves
  aliases under `$app_data_root$` unresolved.
- **Search plan** — `XraySearchPlan` is the ordered search declared before anything is opened: `open` mounts it into a
  VFS of its own, `open_in` into a snapshot built with a cache policy or a read trace, `mount_into` into one someone
  else holds. `XrayRoots::to_search_plan` plans what a request names. A root that is not there is an empty step reported
  as skipped.

## Layer roots and read

```rust,no_run
use xrf_vfs::{XrayMountMode, XraySearch, XraySearchPlan};

# fn main() -> xrf_error::XrfResult {
// A mod tree in front of the installation it modifies: the mod wins every collision, the game answers the rest.
let search: XraySearch = XraySearchPlan::new()
  .with_root_mode("mod", "C:\\work\\my_mod", XrayMountMode::Directory)?
  .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Installation)?
  .ignoring(&[String::from("textures\\wip")])?
  .open()?;

if let Some(asset) = search.find("configs\\system.ltx")? {
  // Prefer read_asset once a lookup produced the asset: it reads the copy found instead of looking the path up again.
  let bytes: Vec<u8> = search.read_asset(&asset)?;

  // to_physical_path is None for an archived asset — reading must go through the search.
  println!("{} bytes from {:?}", bytes.len(), asset.to_physical_path());
}

// Report sources that could not be opened.
for skipped in search.list_skipped_sources() {
  eprintln!("skipped {}: {}", skipped.path.display(), skipped.reason);
}
# Ok(())
# }
```

`ignoring` hides logical prefixes in every loose directory the plan mounts; archive volumes are unaffected. A source
that fails to open is logged, recorded on the snapshot and reported by its step, so a partly readable installation
still resolves what it can. A recorded failure is settled: mounting the same plan again does not retry it until
`XraySearchPlan::forget_skipped_in` forgets it.

## Enumerate assets

```rust,no_run
use xrf_vfs::{XrayAssetType, XrayMountMode, XraySearch, XraySearchPlan};

# fn main() -> xrf_error::XrfResult {
let search: XraySearch = XraySearchPlan::new()
  .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Auto)?
  .open()?;

let textures = search.list_assets(XrayAssetType::Dds); // every .dds, wherever it lives
let top_level = search.list_children("")?; // one directory level, for a tree view
let configs = search.clone().with_prefix("configs")?.list_files(); // one subtree, no sizes

println!(
  "{} textures, {} configs, {} top-level folders",
  textures.len(),
  configs.len(),
  top_level.directories.len()
);
# Ok(())
# }
```

Every listing holds one entry per logical path, the copy the earliest step answers with. From cheapest to heaviest:
`list_files` returns winners without sizes; `list_entries` pairs each winner with its size, which costs a metadata
lookup per loose file; `list_shadowing_entries` adds the copies each winner hides, each with its own size. These three
and `list_children` are ordered by logical path; `list_assets` keeps step order. `list_collisions` reports names that
fold to one logical path within a single source, `list_sources` the mounts a search reaches and the step that reaches
each, `list_roots` and `list_containers` the same sources named by root path or by individual volume.

## Resolve engine references

A reference out of a config or a mesh header is not a path: the kind's directory and extension are implied, and it may
carry an authoring extension the engine swaps. `resolve` applies the kind's rules, reports what the lookup came to, and
treats a `*` mask as one reference with several answers.

```rust,no_run
use xrf_vfs::{XrayAssetType, XrayMountMode, XrayResolution, XraySearch, XraySearchPlan};

# fn main() -> xrf_error::XrfResult {
let search: XraySearch = XraySearchPlan::new()
  .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Auto)?
  .open()?;

let texture: XrayResolution = search.resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")?; // textures\wpn\wpn_ak74.dds
let motions = search.resolve(XrayAssetType::Omf, "wpn\\wpn_ak74_*")?.into_assets(); // every matching animation set

println!("{:?} answered, {} motions", texture.get_step(), motions.len());
# Ok(())
# }
```

Only kinds with one home resolve: `Ogf` and `Omf` under `meshes`, `Dds` and `Thm` under `textures`, `Ogg` under
`sounds`, `Ltx` under `configs`, `Script` under `scripts`, and `Ppe` under `anims` (`XrayAssetType::get_rules`). A `Dds`
reference naming `.tga`, `.bmp` or `.ogm` resolves to the `.dds` beside it. A mask holds exactly one `*`.
`resolve_with_fallback` tries a second reference and reports `Substituted` when only that one answered; `find_beside`
and `locate_beside` look for a reference under a level directory instead of the kind's home, as the engine's `$level$`
does.

## Change files

A change goes through a transaction made from a search and is addressed through that search's steps. Committing
writes to disk and answers the next snapshot with an `XrayChangeSet`; the snapshot committed against stays as it was.

```rust,no_run
use xrf_vfs::{XrayChangeSet, XrayInputRecord, XrayMountMode, XraySearch, XraySearchPlan, XrayTransaction, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let search: XraySearch = XraySearchPlan::new()
  .with_root_mode("mod", "C:\\work\\my_mod", XrayMountMode::Directory)?
  .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Installation)?
  .open()?;

// Record what the edit is based on, so the commit is refused if any of it changed in the meantime.
let record: XrayInputRecord = XrayInputRecord::new();
let original: Vec<u8> = search.clone().with_recording(&record).read("configs\\system.ltx")?;
let mut edited: Vec<u8> = original;

edited.extend_from_slice(b"\n; edited\n");

let transaction: XrayTransaction = XrayTransaction::new(&search)
  .with_write("configs\\system.ltx", edited)?
  .with_expected(&record);
let (next, changes): (XrayVfs, XrayChangeSet) = search.get_vfs().commit(&transaction)?;

// Later reads go through a search over the new snapshot; the mounts and steps carry over unchanged.
let search: XraySearch = XraySearch::new(next).with_steps(search.get_steps().to_vec());

println!("revision {}: {} change(s)", changes.get_revision().get(), changes.get_changes().len());
println!("{:?}", search.read_size("configs\\system.ltx"));
# Ok(())
# }
```

- A path a writable loose mount answers for is written in place. A path only archived, or not present, is created in
  the first writable loose mount the search reaches, where it then wins. A delete needs a loose winner.
- Every file is staged beside its target and synced, then swapped in; a failed swap puts every file back, so a commit
  changes every file or none.
- Each changed file takes the new project revision, and so does every directory above a created or deleted file. A
  snapshot's retained values for untouched files survive the change.
- `XrayVfs::apply` takes changes already on disk (from a file watcher, say) and writes nothing. An archive mount refuses
  a change with `NotImplemented`.
- A loose asset's host path is `XrayAsset::to_writable_path`, which refuses an archived winner by name.

## Share one snapshot

`XrayVfsHost` owns the current snapshot for a long-running process such as the desktop app. `search(&roots)` plans an
`XrayRoots` once, mounts only what the newest snapshot lacks, and answers a search; `search_fresh` plans again and
retries sources that failed to open, for an open a person makes. Mounts and changes (`commit`, `apply`) are serialized
on one lock and publish the next snapshot whole, so a search never waits for either and keeps reading the snapshot it
was given. The snapshot's cache policy and read trace, set on the first `XrayVfs` passed to `XrayVfsHost::new`, carry
to every later snapshot.

Further flows are documented on their types: recording what a search reads on `XraySearch::with_recording` and
`XrayVfs::answer_input`, retention on `XrayCachePolicy` and `XraySearch::read_parsed`, and out-of-crate sources on
`XrayAssetSource`.

## Errors

| Operation                           | Absent asset                  | Failure                                        |
| ----------------------------------- | ----------------------------- | ---------------------------------------------- |
| `find`, `find_beside`               | `Ok(None)`                    | `Err` — invalid path, or a kind with no home   |
| `locate`, `resolve`                 | `Ok(XrayResolution::Missing)` | `Err` — invalid path or reference              |
| `read`, `read_asset`                | `Err` (`NotFound` variant)    | `Err` — the source's own read error            |
| `read_size`, `read_recorded_crc`    | `None`                        | `None` — a gate discards the difference        |
| `XraySearchPlan::with_root_mode`    | Empty step, reported skipped  | `Err` — an existing root cannot be planned     |
| `XraySearchPlan::open`              | n/a                           | `Err` only when a plan cannot be mounted       |
| `XrayVfs::commit`, `XrayVfs::apply` | n/a                           | `Err`, with nothing changed                    |

`locate` and `resolve` answer `NoScope` instead of `Missing` when no step selects a mounted source; `resolve` also fails
for a kind with no home and a mask with more than one `*`. `with_root_mode` fails for `Installation` on a path holding
no fsgame file or for an fsgame file that cannot be read; `open` reports an unopenable source as skipped instead. A
commit fails for a stale expectation, a path no writable mount can take, or a failed swap. `NotFound` is reserved for
absence, so a consumer can tell "the asset is not here" from "the source holding it failed" without parsing messages.
`XraySearch::open` and `read_shared` are declared but always answer `NotImplemented`: every read is whole.

## Module map

The crate is grouped by the question each part answers; every module is private and its supported names are
re-exported from the crate root:

- `path` — what an engine identity is, and the only place separators and case are decided.
- `asset` — what a resolved asset is, plus the per-kind rules that turn a reference into one.
- `fsgame` — the declaration file an installation describes its own layout with.
- `source` — the mountable surface, and the two sources the engine itself has.
- `mount` — composing sources into a mounted order, and planning one from a path.
- `search` — how a VFS snapshot is read: the ordered search, its steps and what its lookups answer.
- `vfs` — the VFS snapshot itself (mounts, revisions, what it retains) and the host owning the current one.
- `change` — changing what a snapshot holds: a transaction committed to disk, or changes already there applied.
- `input` — which inputs a recording search resolved, and the revision each had then.
- `cache` — what a snapshot keeps after parsing an asset, the policy deciding what is kept, and the coordination that
  keeps two readers from parsing one asset twice.
- `trace` — what a snapshot actually read, when a caller asked to be told.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-vfs
cargo test --locked --doc -p xrf-vfs
```

See the [search](src/search/xray_search.rs), [search plan](src/search/xray_search_plan.rs),
[snapshot](src/vfs/xray_vfs.rs), [host](src/vfs/xray_vfs_host.rs), [commit](src/change/xray_commit.rs), and
[public exports](src/lib.rs).

## Related crates

- [xrf-archive](../xrf-archive/README.md) — the `.db` volume format, which the archive source reads; it lives below
  this crate.
- [xrf-pack](../xrf-pack/README.md) — packing and extraction.
- [xrf-ltx](../xrf-ltx/README.md), [xrf-gamedata](../xrf-gamedata/README.md) — config and gamedata projects over a
  search.
