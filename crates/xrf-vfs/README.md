# xrf-vfs

Indexes X-Ray assets and layers their physical sources in a virtual file system.

Plan a search over the places to read, open it, and read through it. Lookups span loose files and archive entries in
priority order. Installation plans read that order from `fsgame.ltx`, so an override resolves to the file the engine
would load.

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

- **Logical path** — an engine identity: lower case, `\`-separated, no `.` or `..`. `XrayLogicalPath` is the typed form;
  every lookup accepts a raw `&str` and normalizes it, so `Configs/System.LTX` names the same asset.
- **VFS snapshot** — `XrayVfs` holds mounted sources (a loose directory or a `.db` volume set at a logical base) in
  priority order, the revisions of what they hold, and what it retains after parsing. It answers for its mounts; it is
  read only through a search.
- **Search** — `XraySearch` is an ordered list of steps over one snapshot it owns, first hit winning: what PhysicsFS and
  Source's file system call a search path. Each step is a label and an `XrayLookupScope` (which mounts, which logical
  subtree). A search is cheap to clone and can record every input it reads.
- **Mode** — `XrayMountMode` says how a caller's path becomes mounts: `Auto` detects an installation at exactly the
  given path, `Directory` forces a loose root, `Volumes` mounts every `.db` volume beneath the path rather than only
  the ones directly under it, `Installation` requires an fsgame file, and `ContainingInstallation` searches upward for
  one, falling back to a loose root when none exists.
- **fsgame file** — the layout file that declares an installation, found by `FsgameFile::find`. Every OpenXRay-family
  engine reads `fsgame.ltx`; each Enhanced Edition's engine reads only its own game's name instead (`fsgame_soc.ltx`,
  `fsgame_cs.ltx`, `fsgame_cop.ltx`) and supplies `$fs_root$` and `$app_data_root$` itself. A directory is searched in
  `FsgameFile::FILE_NAMES` order, `fsgame.ltx` first.
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
  // Prefer read_asset once a lookup produced the asset: it reads from the mount that answered.
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

Listings are ordered by logical path. `list_files` returns winners without sizes; `list_entries` pairs each winner with
its size; `list_shadowing_entries` adds the copies each winner hides. `list_collisions` reports conflicting names within
mounts, `list_sources` the mounts a search reaches and the step that reaches each.

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

Advanced flows are documented on their types: recording what a search reads on `XraySearch::with_recording` and
`XrayVfs::answer_input`, retention on `XrayCachePolicy`, and out-of-crate sources on `XrayAssetSource`.

## Errors

| Operation                 | Absent asset                 | Failure                                                                                           |
| ------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------- |
| `find`, `find_beside`     | `Ok(None)`                   | `Err` — invalid path or kind                                                                      |
| `resolve`, `locate`       | `Ok(XrayResolution::Missing)` | `Err` — invalid reference or kind                                                                 |
| `read`, `read_asset`      | `Err` (`NotFound` variant)   | `Err` — the source's own read error                                                               |
| `read_size`               | `None`                       | `None` — a size gate discards the difference                                                      |
| `XraySearchPlan::open`    | n/a                          | `Err` only when planning fails; an unopenable source is skipped and reported by `list_skipped_sources` |

`NotFound` is reserved for absence, so a consumer can tell "the asset is not here" from "the source holding it failed"
without parsing messages.

## Related crates

- [xrf-archive](../xrf-archive/README.md) — the `.db` volume format.
- [xrf-pack](../xrf-pack/README.md) — packing and extraction.
- [xrf-ltx](../xrf-ltx/README.md), [xrf-gamedata](../xrf-gamedata/README.md) — config and gamedata projects over a search.
