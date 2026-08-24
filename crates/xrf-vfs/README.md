# xrf-vfs

Indexes X-Ray assets and layers their physical sources in a virtual file system.

`XrayVfs::open` is the front door: a mode and a path become something you can resolve and read against. Everything a
consumer needs is exported from the crate root, so `use xrf_vfs::XrayVfs` is the import regardless of how the inside is
arranged. The root exports only types; helpers and constants hang off the type that owns their concept.

## Why not `std::fs`

The engine does not see a disk — it sees ordered sources. An installation's `fsgame.ltx` declares directories of loose
files and directories of `.db` archive volumes, the engine registers them all, and a later registration shadows an
earlier one. Many assets have no file of their own: on Anomaly, meshes and level bundles come out of `db\` volumes.
Identities are engine paths — lower case, `\`-separated on every platform — not host paths.

`std::fs` answers "what is on disk". This crate answers "what would the engine load, and from where":

- One lookup spans loose trees and archive volumes, first hit wins — the same winner `CLocatorAPI` picks.
- `XrayLogicalPath` keeps engine identities apart from host paths.
- Reading goes through the VFS, so an asset resolves the same whether it is loose or archived.

## Quickstart

```rust,no_run
use xrf_vfs::{XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let vfs: XrayVfs = XrayVfs::open(XrayMountMode::Auto, "C:\\Games\\Anomaly")?;
let bytes: Vec<u8> = vfs.read_bytes("configs\\system.ltx")?;
# Ok(())
# }
```

## Concepts

- **Logical path** — an engine identity: lower case, `\`-separated, no `.` or `..`. `XrayLogicalPath` is the typed form;
  every lookup accepts a raw `&str` and normalizes it, so `Configs/System.LTX` names the same asset.
- **Mount** — one source (a loose directory or a `.db` volume set) at a logical base. `XrayVfs` searches its mounts in
  order; the first mount holding a path wins and shadows the rest.
- **Mode** — `XrayMountMode` says how a caller's path becomes mounts: `Auto` detects an installation at exactly the
  given path, `Directory` forces a loose root, `Installation` requires an `fsgame.ltx`, and `ContainingInstallation`
  searches upward for one. Every tool surface exposes this same vocabulary, so `--source` means the same thing
  everywhere.
- **Plan** — `XrayMountPlan` is the inspectable list of sources a mode decided to mount, before anything is opened.
  Reach for it only to inspect or chain plans; `XrayVfs::open` plans and mounts in one call.
- **Scope** — `XrayLookupScope` narrows where one lookup may look: which mounts, and which logical subtree. Lookups on
  `XrayVfs` span everything; state a scope once with `vfs.scoped(&scope)` and the view exposes the same operations,
  narrowed.

## Open an installation or a loose tree

```rust,no_run
use xrf_vfs::{XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
// An installation: fsgame.ltx declares the layout, archives and gamedata both mount.
let installation: XrayVfs = XrayVfs::open(XrayMountMode::Installation, "C:\\Games\\Anomaly")?;

// A loose gamedata tree, ignoring any fsgame.ltx beside it.
let tree: XrayVfs = XrayVfs::open(XrayMountMode::Directory, "C:\\work\\gamedata")?;

println!("{} + {} mounts", installation.get_mounts().len(), tree.get_mounts().len());

// Mounting is tolerant: a volume that fails to open is skipped and recorded, so report these too —
// a mount that silently vanished reads as content that is silently missing.
for skipped in installation.get_skipped_mounts() {
  eprintln!("skipped {}: {}", skipped.path.display(), skipped.reason);
}
# Ok(())
# }
```

## Read bytes, check existence

Absence is not an error for a lookup, and is a `NotFound` error for a read — see [Errors](#errors).

```rust,no_run
use xrf_vfs::{XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let vfs: XrayVfs = XrayVfs::open(XrayMountMode::Auto, "C:\\Games\\Anomaly")?;

if let Some(asset) = vfs.find("configs\\system.ltx")? {
  // Prefer read_asset once a lookup produced the asset: it reads from the mount that answered.
  let bytes: Vec<u8> = vfs.read_asset_bytes(&asset)?;

  // to_physical_path is None for an archived asset — reading must go through the VFS.
  println!("{} bytes from {:?}", bytes.len(), asset.to_physical_path());
}
# Ok(())
# }
```

## Enumerate assets

```rust,no_run
use xrf_vfs::{XrayAssetType, XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let vfs: XrayVfs = XrayVfs::open(XrayMountMode::Auto, "C:\\Games\\Anomaly")?;

let textures = vfs.list_entries_of_type(XrayAssetType::Dds); // every .dds, wherever it lives
let particle_packs = vfs.list_entries_with_suffix("particles.xr")?; // named by convention, not extension
let top_level = vfs.list_children("")?; // one directory level, for a tree view

println!(
  "{} textures, {} particle packs, {} top-level folders",
  textures.len(),
  particle_packs.len(),
  top_level.directories.len()
);
# Ok(())
# }
```

Listings are winners only, ordered by logical path; `list_entries_all` keeps shadowed copies for override auditing, and
`list_collisions` reports files a mount holds but cannot reach.

## Resolve engine references

A reference out of a config or a mesh header is not a path: the kind's directory and extension are implied, and it may
carry an authoring extension the engine swaps. `resolve` applies the kind's rules; a `*` mask names a set.

```rust,no_run
use xrf_vfs::{XrayAssetType, XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let vfs: XrayVfs = XrayVfs::open(XrayMountMode::Auto, "C:\\Games\\Anomaly")?;

let texture = vfs.resolve_dds_texture("wpn\\wpn_ak74")?; // textures\wpn\wpn_ak74.dds
let visual = vfs.resolve_ogf("weapons\\ak74\\wpn_ak74_hud")?; // meshes\weapons\ak74\wpn_ak74_hud.ogf
let motions = vfs.resolve_all(XrayAssetType::Omf, "wpn\\wpn_ak74_*")?; // every matching animation set

println!("{:?} {:?} {}", texture.is_some(), visual.is_some(), motions.len());
# Ok(())
# }
```

## Narrow with a scope

State the scope once; the view exposes the same operations.

```rust,no_run
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};

# fn main() -> xrf_error::XrfResult {
let vfs: XrayVfs = XrayVfs::open(XrayMountMode::Auto, "C:\\Games\\Anomaly")?;

// Only the configs subtree — a config project and an asset lookup share one VFS and differ only in scope.
let configs: XrayLookupScope = XrayLookupScope::all().with_prefix("configs")?;
let system: Vec<u8> = vfs.scoped(&configs).read_bytes("configs\\system.ltx")?;

// Other narrowings: only writable mounts, only named mounts, only one storage kind.
let writable: XrayLookupScope = XrayLookupScope::writable();
println!("{} bytes, {} writable entries", system.len(), vfs.scoped(&writable).list_entries().len());
# Ok(())
# }
```

Advanced flows are documented on their types: layering a mod tree over an installation on `XrayMountPlan` (`behind`,
`ignoring`), write-back on `XrayVfs::write` and `XrayVfs::write_override`, and out-of-crate sources on
`XrayAssetSource`.

## Errors

| Operation            | Absent asset               | Failure                                                                                              |
|----------------------|----------------------------|------------------------------------------------------------------------------------------------------|
| `find`, `resolve`    | `Ok(None)`                 | `Err` — invalid path or kind                                                                         |
| `read`, `read_asset` | `Err` (`NotFound` variant) | `Err` — the source's own read error                                                                  |
| `read_size`          | `None`                     | `None` — a size gate discards the difference                                                         |
| `open`, `mount_plan` | n/a                        | `Err` only when planning fails; an unopenable source is skipped and recorded on `get_skipped_mounts` |

`NotFound` is reserved for absence, so a consumer can tell "the asset is not here" from "the source holding it failed"
without parsing messages.

## Related crates

- `xrf-archive` — the `.db` volume format underneath this crate's archive source.
- `xrf-pack` — pack and unpack tooling over both.
- `xrf-ltx`, `xrf-gamedata` — config and gamedata projects that read through this VFS, so archived installations verify
  as readily as loose trees.
