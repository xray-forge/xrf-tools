# xrf-extension

Splits an extension off an engine or host file name, and names the extensions X-Ray game data and XRF tooling actually
ship. The bottom of the workspace: nothing here knows what an extension is *for*.

## Read the extension of a name

```rust
use xrf_extension::{XrayExtension, XrayExtensionOf};

// The engine ships `shaders\r1\.s`, a Lua script whose whole name is its extension.
assert_eq!(
    XrayExtensionOf::of("shaders\\r1\\.s"),
    XrayExtensionOf::Known(XrayExtension::S)
);

// A spelling nothing here models keeps its text, so a viewer can still say what it refused.
assert_eq!(XrayExtensionOf::of("textures\\wpn\\ak74.psd"), XrayExtensionOf::Unknown("psd"));

assert_eq!(XrayExtensionOf::of("levels\\l01_escape\\level"), XrayExtensionOf::None);
```

One call answers both questions a caller has: logic wants the variant, and a person-facing message wants the text of a
spelling the variant list does not cover. Nothing is allocated, because this runs once per entry of a name table that
holds hundreds of thousands of them.

## Compare one extension

```rust
use std::path::Path;

use xrf_extension::{XrayExtension, get_path_extension};

assert!(XrayExtension::Ltx.matches("configs\\SYSTEM.LTX"));

// The door for a caller holding a host path, which never converts the whole path to text.
assert!(XrayExtension::Ltx.matches_path(Path::new("configs/system.ltx")));

// The extension is the last dot segment of the last path segment, never a byte suffix.
assert!(!XrayExtension::Ltx.matches("notes.myltx"));
assert!(!XrayExtension::Ltx.matches("configs\\weapons.old\\readme"));

// The one raw split left exported, for the numbered `db0`/`xdb1` volume family no vocabulary can hold.
assert_eq!(get_path_extension(Path::new("db\\game.db0")), Some("db0"));
```

## What belongs in the vocabulary

One variant per real on-disk spelling, added only with evidence from a tree or from a reader that loads it. Membership
is not a policy: which extensions are read as text, compressed, or skipped stays with the crate that decides it, because
the same spelling answers differently in each. `tga` is an authoring source for a texture and a file `xrCompress` skips,
and merging those two lists is how one of them becomes wrong.

Build-tool detritus a packer's skip list names — `vcproj`, `sln`, `rc` — is not vocabulary. That list pattern-matches
prefixes as well as spellings and stays string-based where it is.

Neither are archive volumes. `db`, `db0`..`db9`, `xdb`, `xdb0`..`xdb9` are one numbered family whose extension carries
an index, so `xrf-pack`'s `ArchiveVolumeExtension` owns the stem and `xrf-archive` recognizes a volume by prefix over
the split extension. A closed vocabulary cannot hold a family; do not add `Db` to complete the list.

Consumers name the variant directly — `XrayExtension::Ltx`, not a local `const LTX_EXTENSION` aliasing it. Eleven such
aliases were deleted on 2026-09-13; each added a name, an import and a hop while hiding nothing. A named constant here
earns its name only when it names a *decision*, which is what the policy sets do.

## Checks

Run `cargo test --locked -p xrf-extension` from the repository root. This crate splits and names extensions; it does not
decide what any of them is for. Which extensions a consumer reads as text, compresses, or skips is that consumer's
policy list, and `XrayAssetType` in [xrf-vfs](../xrf-vfs/README.md) owns the engine-tree knowledge on top.

See the [vocabulary](src/xray_extension.rs), the [answer type](src/xray_extension_of.rs), and the
[splitter](src/file_extension.rs).
