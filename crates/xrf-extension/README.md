# xrf-extension

Splits an extension off an engine or host file name, and names the extensions X-Ray game data and XRF tooling actually
ship. It is the bottom of the workspace and holds no policy: which extensions a consumer reads as text, compresses, or
skips is that consumer's list, and [xrf-vfs](../xrf-vfs/README.md)'s `XrayAssetType` owns the engine-tree knowledge on
top.

An extension question has three layers. This crate holds the first two, together, because keeping them apart is what
let the workspace grow three answers to the same question; the third, what kind of asset an extension names, is
`XrayAssetType`. One module each:

- `file_extension` — how an extension comes off a name or a host path, and the one comparison of an extension against
  a spelling. The only raw split exported is `get_path_extension`.
- `xray_extension` — which extensions exist: `XrayExtension`, a closed vocabulary. `parse` folds case, `as_str` and
  `Display` give the on-disk spelling, which is also the serde name, `ALL` lists every member in declaration order, and
  `get_media_type` names the IANA type a webview can render or play as it stands (`None` for most, including `dds` and
  `tga`).
- `xray_extension_of` — the one door from a name to what its extension turned out to be: `XrayExtensionOf`.

## Read the extension of a name

```rust
use xrf_extension::{XrayExtension, XrayExtensionOf};

// The engine ships `shaders\r1\.s`, a Lua script whose whole name is its extension.
assert_eq!(
  XrayExtensionOf::of("shaders\\r1\\.s"),
  XrayExtensionOf::Known(XrayExtension::S)
);

// A spelling nothing here models keeps its text, so a viewer can still say what it refused.
assert_eq!(XrayExtensionOf::of("textures\\wpn\\ak74.PSD"), XrayExtensionOf::Unknown("PSD"));

assert_eq!(XrayExtensionOf::of("levels\\l01_escape\\level"), XrayExtensionOf::None);

// A known extension is named by its declared spelling, whatever case it was authored in.
assert_eq!(XrayExtensionOf::of("textures\\A.DDS").as_str(), Some("dds"));
assert_eq!(XrayExtensionOf::of("configs\\system.ltx").known(), Some(XrayExtension::Ltx));
```

One call answers both questions a caller has: logic wants the variant (`known`), and a person-facing message wants the
text of a spelling the vocabulary does not cover (`Unknown`, as authored). Nothing is allocated, because this runs once
per entry of a name table that holds hundreds of thousands of them.

The extension is whatever follows the last dot of the last path segment, `\` or `/`. Unlike `Path::extension`, a name
beginning with a dot is an extension, not a hidden file: nothing in game data is a Unix dotfile. `of_path` reads a host
path's file name alone, so a parent directory that is not valid Unicode cannot hide a valid extension; a file name that
is not valid Unicode has none.

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

Comparison is undotted and ASCII case-insensitive. Use `matches`, `matches_path`, or `XrayExtensionOf` for game data
rather than `Path::extension`, which disagrees about leading-dot names.

## What belongs in the vocabulary

One variant per real on-disk spelling, added only with evidence from a tree or from a reader that loads it; `anm` and
`anm1` are two members, not one normalized to the other. Declare variants alphabetically with their exact spelling
(`ps_static`, not `psStatic`): declaration order is `ALL`, and the `typescript-bindings` feature mirrors the enum into
the frontend's generated types.

Membership is not a policy: which extensions are read as text, compressed, or skipped stays with the crate that decides
it, because the same spelling answers differently in each. `tga` is an authoring source for a texture and a file
`xrCompress` skips, and merging those two lists is how one of them becomes wrong.

Build-tool detritus a packer's skip list names — `vcproj`, `sln`, `rc` — is not vocabulary. That list pattern-matches
prefixes as well as spellings and stays string-based where it is.

Neither are archive volumes. `db`, `db0`..`db9`, `xdb`, `xdb0`..`xdb9` are one numbered family whose extension carries
an index, so `xrf-pack`'s `ArchiveVolumeExtension` owns the stem and `xrf-archive` recognizes a volume by prefix over
the split extension. A closed vocabulary cannot hold a family; do not add `Db` to complete the list.

Consumers name the variant directly — `XrayExtension::Ltx`, not a local `const LTX_EXTENSION` aliasing it: an alias adds
a name, an import and a hop while hiding nothing. A named constant earns its name only when it names a decision, which
is what a consumer's policy set does.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-extension
```

The README is the crate documentation, so its Rust examples run as doctests. See the
[vocabulary](src/xray_extension.rs), the [answer type](src/xray_extension_of.rs), and the
[splitter](src/file_extension.rs).
