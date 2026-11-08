# xrf-vfs

Finds game files the way the engine does: across loose folders, archives and mod trees, in priority order.

The engine never opens a game file by its host path. It asks for an engine path such as `configs\system.ltx`, and its
file system answers from whichever mounted source holds it first: a loose `gamedata` tree, an archive volume set, or a
mod tree in front of both. This crate reproduces that lookup.

1. Plan a search over the places to read.
2. Open it.
3. Read through it.

Installation plans read the mount order from `fsgame.ltx`, so an override resolves to the file the engine would load.
Format crates such as `xrf-ltx` and `xrf-gamedata` read through a search; the binaries own mounting.

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

An SDK's rawdata opens the same way, through the layout its editors read:

```rust,no_run
use xrf_vfs::{XraySdkInstallation, XraySearch};

# fn main() -> xrf_error::XrfResult {
let sdk: XraySdkInstallation = XraySdkInstallation::read("C:\\SDK\\editors")?;
let rawdata: XraySearch = sdk.to_rawdata_search_plan()?.open()?;
let objects: String = sdk.get_rawdata_prefix("$objects$")?.unwrap_or_default();
let bytes: Vec<u8> = rawdata.read(&format!("{objects}\\editor\\axis.object"))?;
# Ok(())
# }
```

## Concepts

| Term         | Type              | In short                                                   |
| ------------ | ----------------- | ---------------------------------------------------------- |
| Logical path | `XrayLogicalPath` | An engine path: lower case, `\`-separated                  |
| Search plan  | `XraySearchPlan`  | The ordered places to read, declared before anything opens |
| VFS snapshot | `XrayVfs`         | The mounted sources, frozen while in use                   |
| Search       | `XraySearch`      | Ordered steps over one snapshot; the first hit wins        |
| Mount mode   | `XrayMountMode`   | How a caller's path turns into mounts                      |

### Logical path

An engine identity: lower case, `\`-separated, with no empty, `.` or `..` component and no colon. Every lookup accepts
a raw `&str` and normalizes it, so `Configs/System.LTX` names the same asset.

### Search plan

The ordered search, declared before anything is opened.

- `open` mounts it into a VFS of its own.
- `open_in` mounts it into a snapshot built with a cache policy or a read trace.
- `mount_into` mounts it into a snapshot someone else holds.
- `XrayRoots::to_search_plan` plans what a request names.
- A root that is not there becomes an empty step, reported as skipped.

### VFS snapshot

`XrayVfs` holds the mounted sources (a loose directory, or a `.db` volume set at a logical base) in priority order, the
revisions of what they hold, and what it keeps after parsing.

- It is read only through a search, which shares it behind an `Arc`, so a snapshot in use never changes.
- `commit` and `apply` return the next snapshot instead.
- A host mounts into a fork (`XrayVfs::fork`, which shares sources and cache).

### Search

`XraySearch` is an ordered list of steps over one snapshot it owns, the first hit winning: what PhysicsFS and Source's
file system call a search path.

- Each step is a label and an `XrayLookupScope`: which mounts, in what order, and which logical subtree.
- A search has no lifetime, is cheap to clone, and can record every input it reads.

### Mount modes

| Mode                     | The path is mounted as                                                                                                                                   |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Auto` (default)         | An installation if it holds an fsgame file; one volume if it is one; a volume set if it holds volumes directly; else a loose root. Never searches upward |
| `Directory`              | A loose root                                                                                                                                             |
| `Volumes`                | Every `.db`/`.xdb` volume beneath it, recursively                                                                                                        |
| `Installation`           | An installation; an fsgame file is required                                                                                                              |
| `ContainingInstallation` | The nearest installation above it, or a loose root when there is none                                                                                    |

### fsgame files

The fsgame file declares an installation's layout; `FsgameFile::find` finds it.

- Every OpenXRay-family engine reads `fsgame.ltx`. Each Enhanced Edition reads only its own game's name
  (`fsgame_soc.ltx`, `fsgame_cs.ltx`, `fsgame_cop.ltx`) and supplies `$fs_root$` and `$app_data_root$` itself.
- A directory is searched in `FsgameFile::FILE_NAMES` order, `fsgame.ltx` first.
- `$fs_root$` is anchored at the file's directory. A root column naming a directory is taken as the engine takes it: an
  absolute one as is, a relative one under the file's directory (the working directory the engine opened it from).
- A root starting with `\` but no drive, and aliases under an engine-supplied `$app_data_root$`, stay unresolved.

### SDK installations

`XraySdkInstallation` reads the `fs.ltx` in an SDK's editors directory. It uses the same format to lay out rawdata (the
SDK's authoring tree) under `$server_data_root$`, beside gamedata.

- Rawdata is searched separately from gamedata, since both hold `textures\`, `sounds\` and `levels\`.
- `to_rawdata_search_plan` mounts it as a root of its own.
- `get_rawdata_prefix` gives an editor alias's logical path inside it: `objects` for `$objects$`.

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

- `ignoring` hides logical prefixes in every loose directory the plan mounts; archive volumes are unaffected.
- A source that fails to open is logged, recorded on the snapshot, and reported by its step, so a partly readable
  installation still resolves what it can.
- A recorded failure stays settled: mounting the same plan again does not retry it until
  `XraySearchPlan::forget_skipped_in` clears it.

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

Every listing holds one entry per logical path: the copy the earliest step answers with.

| Listing                  | Returns                                                 | Order        |
| ------------------------ | ------------------------------------------------------- | ------------ |
| `list_files`             | Winners, without sizes (cheapest)                       | Logical path |
| `list_entries`           | Winners with sizes; one metadata lookup per loose file  | Logical path |
| `list_shadowing_entries` | Also the copies each winner hides, with their own sizes | Logical path |
| `list_children`          | One directory level                                     | Logical path |
| `list_assets`            | Every asset of one kind                                 | Step order   |

About the sources:

- `list_collisions` reports names that fold to one logical path within a single source.
- `list_sources` lists the mounts a search reaches and the step that reaches each.
- `list_roots` and `list_containers` name the same sources by root path, or by individual volume.

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

Only kinds with one home resolve (`XrayAssetType::get_rules`):

| Kind         | Home       |
| ------------ | ---------- |
| `Ogf`, `Omf` | `meshes`   |
| `Dds`, `Thm` | `textures` |
| `Ogg`        | `sounds`   |
| `Ltx`        | `configs`  |
| `Script`     | `scripts`  |
| `Ppe`        | `anims`    |

- A `Dds` reference naming `.tga`, `.bmp` or `.ogm` resolves to the `.dds` beside it.
- A mask holds exactly one `*`.
- `resolve_with_fallback` tries a second reference and reports `Substituted` when only that one answered.
- `find_beside` and `locate_beside` look under a level directory instead of the kind's home, as the engine's `$level$`
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

**Where a write goes.**

- A path a writable loose mount answers for is written in place.
- A path that is only archived, or absent, is created in the first writable loose mount ahead of the copy that answers,
  and refused when there is none.
- `with_write_in_place` refuses anything but a writable loose winner. A delete needs a loose winner.
- `with_file_write` names a file on disk instead, written in every writable loose mount whose root holds it.
- An archive mount refuses a change with `NotImplemented`.
- `XrayAsset::to_writable_path` gives a loose asset's host path, and refuses an archived winner by name.

**How it is written.**

1. Every file is staged beside its target and synced.
2. A journal naming them is written beside the first target.
3. Each file is swapped in. A replaced or deleted file is first kept as a hard link, so its target is never missing.
4. A failed swap puts every file back.

A crash leaves the journal. The next mount of that directory rolls the change back, or forward if every swap was done.
Staged and kept siblings (`.<name>.xrf-…`) are never indexed. A crash while staging leaves hidden staged siblings that
nothing removes.

**Revisions.**

- Each changed file, and every directory above a created or deleted file, takes the change's project revision. The
  snapshot's lineage issues it, so two changes branched from one snapshot never share one.
- A snapshot's retained values for untouched files survive the change.
- A snapshot pins its index and revisions, not loose bytes: an older snapshot reads a changed file's new bytes.

**Applying external changes.** `XrayVfs::apply` takes changes already on disk (from a file watcher, say) and writes
nothing.

## Share one snapshot

`XrayVfsHost` owns the current snapshot for a long-running process such as the desktop app.

- `search(&roots)` plans an `XrayRoots` once, mounts only what the newest snapshot lacks, and returns a search.
- `search_fresh` plans again and retries sources that failed to open, for an open a person makes.
- `search_writable` searches every writable loose mount, for a writer naming files on disk.
- Mounts and changes (`commit`, `apply`, and `apply_changed_on_disk`, which applies each retained loose file changed
  behind every reader) are serialized on one lock and publish the next snapshot whole. A search never waits for them
  and keeps reading the snapshot it was given.
- The cache policy and read trace set on the first `XrayVfs` passed to `XrayVfsHost::new` carry to every later
  snapshot.

More flows are documented on their types:

- recording what a search reads: `XraySearch::with_recording`, `XrayVfs::answer_input`;
- retention: `XrayCachePolicy`, `XraySearch::read_parsed`;
- sources from other crates: `XrayAssetSource`.

## Errors

| Operation                           | Absent asset                  | Failure                                      |
| ----------------------------------- | ----------------------------- | -------------------------------------------- |
| `find`, `find_beside`               | `Ok(None)`                    | `Err` — invalid path, or a kind with no home |
| `locate`, `resolve`                 | `Ok(XrayResolution::Missing)` | `Err` — invalid path or reference            |
| `read`, `read_asset`                | `Err` (`NotFound` variant)    | `Err` — the source's own read error          |
| `read_size`, `read_recorded_crc`    | `None`                        | `None` — a gate discards the difference      |
| `XraySearchPlan::with_root_mode`    | Empty step, reported skipped  | `Err` — an existing root cannot be planned   |
| `XraySearchPlan::open`              | n/a                           | `Err` only when a plan cannot be mounted     |
| `XrayVfs::commit`, `XrayVfs::apply` | n/a                           | `Err`, with nothing changed                  |

More detail:

- `locate` and `resolve` answer `NoScope` instead of `Missing` when no step selects a mounted source.
- `resolve` also fails for a kind with no home, and a mask with more than one `*`.
- `with_root_mode` fails for `Installation` on a path with no fsgame file, or an unreadable one; `open` reports an
  unopenable source as skipped instead.
- A commit fails for a stale expectation, a path no writable mount can take, or a failed swap.
- `NotFound` is reserved for absence, so a consumer can tell "the asset is not here" from "the source holding it
  failed" without parsing messages.
- `XraySearch::open` and `read_shared` are declared but always answer `NotImplemented`: every read is whole.

## Module map

Every module is private; supported names are re-exported from the crate root.

| Module   | Answers                                                                                                 |
| -------- | ------------------------------------------------------------------------------------------------------- |
| `path`   | What an engine identity is; the only place separators and case are decided                              |
| `asset`  | What a resolved asset is, and the per-kind rules that turn a reference into one                         |
| `source` | The mountable surface, and the engine's two sources (`directory/` holds the loose one)                  |
| `mount`  | Ordering sources and planning mounts: `plan/` steps, `root/` caller paths, `fsgame/` layout files       |
| `search` | Reading a snapshot: the ordered search, its `step/`s and scopes, and its `listing/`s                    |
| `vfs`    | The snapshot (mounts, revisions) and its host; `cache/` retention after parsing, `trace/` what was read |
| `change` | Changing a snapshot: a `transaction/` committed through the `disk/` swap, or external changes applied   |
| `input`  | Which inputs a recording search resolved, and their revisions then                                      |

An aspect of `XrayVfs` belonging to one of these lives in that module as `xray_vfs_<aspect>.rs`: plan mounting in
`mount/plan/`, committing and the on-disk check in `change/`, input answers in `input/`.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-vfs
cargo test --locked --doc -p xrf-vfs
```

Sources: the [search](src/search/xray_search.rs), [search plan](src/search/xray_search_plan.rs),
[snapshot](src/vfs/xray_vfs.rs), [host](src/vfs/xray_vfs_host.rs), [commit](src/change/xray_vfs_commit.rs), and
[public exports](src/lib.rs).

## Related crates

- [xrf-archive](../xrf-archive/README.md) — the `.db` volume format, which the archive source reads; it lives below
  this crate.
- [xrf-pack](../xrf-pack/README.md) — packing and extraction.
- [xrf-ltx](../xrf-ltx/README.md), [xrf-gamedata](../xrf-gamedata/README.md) — config and gamedata projects over a
  search.
