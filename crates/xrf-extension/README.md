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
use xrf_extension::{XrayExtension, get_file_extension, has_extension};

assert_eq!(get_file_extension("configs\\system.ltx"), Some("ltx"));
assert!(has_extension("configs\\SYSTEM.LTX", "ltx"));

// The same comparison, reached from the variant, which is what a consumer holding a vocabulary member wants.
assert!(XrayExtension::Ltx.matches("configs\\SYSTEM.LTX"));

// The extension is the last dot segment of the last path segment, never a byte suffix.
assert!(!has_extension("notes.myltx", "ltx"));
assert_eq!(get_file_extension("configs\\weapons.old\\readme"), None);
```

`has_extension` takes the extension undotted, the same spelling `get_file_extension` answers with, and
`XrayExtension::matches` is that same comparison for a caller holding the variant. Three rival rules existed before
this one, and two of them disagreed with the splitter about the engine's own leading-dot names.

## What belongs in the vocabulary

One variant per real on-disk spelling, added only with evidence from a tree or from a reader that loads it. Membership
is not a policy: which extensions are read as text, compressed, or skipped stays with the crate that decides it, because
the same spelling answers differently in each. `tga` is an authoring source for a texture and a file `xrCompress` skips,
and merging those two lists is how one of them becomes wrong.

Build-tool detritus a packer's skip list names — `vcproj`, `sln`, `rc` — is not vocabulary. That list pattern-matches
prefixes as well as spellings and stays string-based where it is.

See the [vocabulary](src/xray_extension.rs), the [answer type](src/xray_extension_of.rs), and the
[splitter](src/file_extension.rs). Run `cargo test --locked -p xrf-extension` from the repository root.
