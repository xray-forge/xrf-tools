# xrf-extension

Splits the extension off a file name and names the extensions X-Ray game data and XRF tooling ship.

It holds no policy. Which extensions a consumer reads as text, compresses or skips is that consumer's own list, and
[xrf-vfs](../xrf-vfs/README.md)'s `XrayAssetType` decides what kind of asset an extension names.

| Module              | Answers                                                      |
| ------------------- | ------------------------------------------------------------ |
| `xray_extension_of` | What a name's extension is: `XrayExtensionOf`                |
| `xray_extension`    | Which extensions exist: `XrayExtension`, a closed vocabulary |
| `file_extension`    | How an extension comes off a name, and how one is compared   |

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

- `known()` gives the variant, for logic. `Unknown` keeps the authored text, for messages.
- Nothing is allocated; this runs once per entry of name tables with hundreds of thousands of entries.
- The extension is whatever follows the last dot of the last path segment, split at `\` or `/`.
- Unlike `Path::extension`, a name starting with a dot is all extension: game data has no Unix dotfiles.
- `of_path` reads only a host path's file name, so a non-Unicode parent directory cannot hide the extension. A
  non-Unicode file name has none.

## The vocabulary

`XrayExtension` is a closed enum:

- `parse` ignores case.
- `as_str` and `Display` give the on-disk spelling, which is also the serde name.
- `ALL` lists every member in declaration order.
- `get_media_type` names the IANA type a webview can render or play as is; `None` for most, including `dds` and `tga`.

## Compare one extension

```rust
use std::path::Path;

use xrf_extension::{XrayExtension, get_path_extension};

assert!(XrayExtension::Ltx.matches("configs\\SYSTEM.LTX"));

// For a host path, without converting the whole path to text.
assert!(XrayExtension::Ltx.matches_path(Path::new("configs/system.ltx")));

// The extension is the last dot segment of the last path segment, never a byte suffix.
assert!(!XrayExtension::Ltx.matches("notes.myltx"));
assert!(!XrayExtension::Ltx.matches("configs\\weapons.old\\readme"));

// The one raw split exported, for the numbered `db0`/`xdb1` volume family no vocabulary can hold.
assert_eq!(get_path_extension(Path::new("db\\game.db0")), Some("db0"));
```

Comparison is without the dot and ignores ASCII case. For game data, use `matches`, `matches_path` or
`XrayExtensionOf` rather than `Path::extension`, which treats leading-dot names differently.

## Adding to the vocabulary

- Add one variant per real on-disk spelling, with evidence from a game tree or a reader that loads it. `anm` and `anm1`
  are two members.
- Declare variants alphabetically with their exact spelling (`ps_static`, not `psStatic`). Declaration order is `ALL`,
  and the `typescript-bindings` feature mirrors the enum into the frontend's types.
- Do not add a consumer's policy here. The same spelling can mean different things: `tga` is a texture source for one
  tool and a file `xrCompress` skips for another.
- Build-tool leftovers in a packer's skip list (`vcproj`, `sln`, `rc`) are not vocabulary; that list stays string-based.
- Archive volumes are not vocabulary either. `db`, `db0`–`db9`, `xdb`, `xdb0`–`xdb9` are a numbered family:
  `xrf-pack`'s `ArchiveVolumeExtension` owns the stem, and `xrf-archive` recognizes volumes by prefix. Do not add `Db`.
- Name variants directly (`XrayExtension::Ltx`) rather than through a local constant alias.

## Checks

Run `cargo test --locked -p xrf-extension` from the repository root.

Sources: the [vocabulary](src/xray_extension.rs), the [answer type](src/xray_extension_of.rs), and the
[splitter](src/file_extension.rs).
