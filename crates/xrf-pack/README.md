# xrf-pack

Packs, unpacks, and patches X-Ray `.db` archive volumes. The volume format and reading it are
[xrf-archive](../xrf-archive/README.md); resolving engine paths across loose files and archives is
[xrf-vfs](../xrf-vfs/README.md). This crate writes: volumes from a directory tree, files out of a volume set or a
mounted search, and patch volumes holding what one tree changed against another.

The operations are unit structs (`ArchivePacker`, `ArchiveUnpacker`, `XrayWorldExtractor`, `ArchivePatcher`) rather
than methods on `ArchiveProject`: they own no state and borrow what they read. Each has a plain entry point and an
`_opt` one taking options for progress, cancellation, and similar run behavior; what to act on is always an argument.

## Pack and unpack

`ArchivePacker` builds volumes from a directory; `ArchiveUnpacker` extracts them into a directory.
`ArchivePackConfig` holds the packing settings. Use `with_config_file` to load selection rules and a header from
`.ltx` (the xrCompress dialect) or `.json`, or `write_config_to_path` to save them. The extension selects the format;
any other is refused. `ArchivePackConfigJson` defines the JSON fields. Paths, volume name, and run options remain in
the calling code.

```rust,no_run
use xrf_archive::ArchiveProject;
use xrf_pack::{ArchivePackConfig, ArchivePacker, ArchiveUnpacker};

# fn main() -> xrf_error::XrfResult {
// Omit with_config_file to use the defaults.
let config: ArchivePackConfig = ArchivePackConfig::new("C:\\work\\gamedata", "C:\\work\\db", "my_mod")
  .with_config_file("C:\\work\\pack.ltx")?;
let packed = ArchivePacker::pack(&config)?;

let project: ArchiveProject = ArchiveProject::new("C:\\Games\\Anomaly\\db")?;
let unpacked = ArchiveUnpacker::unpack(&project, "C:\\work\\unpacked")?;

println!("packed {} files; unpacked {} volumes", packed.files_total, unpacked.archives.len());
# Ok(())
# }
```

Operations are synchronous. On an async executor, run the call on a blocking thread. Use `pack_opt`, `unpack_opt`, or
`extract_directory_opt` with options built by `with_job` for progress and cancellation. A cancelled run returns `Ok`
with `outcome` set to `JobOutcome::Cancelled`, so check the result's `outcome` before treating it as complete.

### Packing

- Archive names are the source-relative paths with `\` separators, keeping their authored case. Two different files
  whose names fold to one engine name (on a case-sensitive host) are refused before any volume exists, every such name
  reported at once; one file reached under two spellings is packed once. A host path that is not valid Unicode is
  refused rather than dropped.
- A run that selects no files is refused. The include, exclude, and skip rules (`is_with_skip_list`, on by default,
  applies xrCompress's hard-coded list) decide the selection; `files_skipped` counts what they left out.
- In the default `ArchivePackMode::Compress`, only `ltx`, `xml`, and `script` files are LZO-compressed, as xrCompress
  does, and only when compression saves enough to keep; `Store` stores everything. A file whose payload equals one
  already written to the same volume costs only a descriptor row (`files_aliased`).
- A set that ends with one volume is named `<name>.db`; a larger one is `<name>.db0`, `<name>.db1`, and so on
  (`.xdb` with `ArchiveVolumeExtension::Xdb`).
- Packing refuses a destination that already holds volumes of the same name unless `ArchivePackOptions::with_force`
  allows it; `ArchivePacker::list_published_volumes` asks before starting. Because an unforced run starts with no volume
  of its set in the destination, a failure or cancellation removes every volume it wrote. A forced run cannot roll back
  or remove stale volumes from a longer previous set; `ArchivePackResult::volumes_opened` lists what it left on disk.

### Unpacking and extraction

- `unpack` writes every entry below the destination at its volume's unpack root (`[header] entry_point`, such as
  `gamedata\`) joined with its name. `extract_directory` writes the entries under one name prefix, matched on whole
  path components without case, relative to that prefix; nothing under it is a `NotFound` error.
- Every entry name is checked before the first write: a name with a `.` or `..` component or a colon fails the run.
- `unpack` and `extract_directory` follow existing symlinks, junctions, and other reparse points below the destination
  root; `ArchiveExtractOptions::with_link_policy(HostLinkPolicy::Refuse)` (`xrf_utils::HostLinkPolicy`) refuses them
  instead, for a destination that is not trusted. `extract_file` writes to the exact path supplied by the caller, which
  may be linked.
- `XrayWorldExtractor::extract_directory` extracts the winning assets from an `XraySearch`, including loose overrides.
  It preserves paths below the selected prefix; an empty prefix extracts every path the search reaches. Shadowed copies
  are omitted, and directories are created only to hold files.
- Extraction can leave completed files behind after failure or cancellation.

## Build a patch

`ArchivePatcher` compares what a patch is built against (`input`) with what it delivers (`target`). `compare` reports
changes without writing; `patch` also writes added and modified entries into new volumes. A patch cannot remove base
entries, so files present only in the base are not reported.

With `target` left out, the patcher splits the one installation's search plan: its archives against its loose
overrides, both read from a single reading of `fsgame.ltx`. An input holding no archive volumes is refused. Call
`with_target` to compare against a separate tree. Publishing refuses a destination inside either side's root, so write
patches elsewhere and deploy them where the installation loads them after its base archives.

```rust,no_run
use xrf_pack::{ArchivePatchConfig, ArchivePatchResult, ArchivePatcher};

# fn main() -> xrf_error::XrfResult {
// The installation's volumes against its own loose gamedata.
let config: ArchivePatchConfig = ArchivePatchConfig::new("C:\\Games\\Anomaly", "C:\\work\\patches", "patch_02");

let preview: ArchivePatchResult = ArchivePatcher::compare(&config)?;

println!("{} added, {} modified", preview.added.len(), preview.modified.len());

let published: ArchivePatchResult = ArchivePatcher::patch(&config)?;

println!("carried {} entries", published.get_carried_count());
# Ok(())
# }
```

Two copies are equal when their sizes and CRC32 match. An archive records its CRC; a loose file is read to compute one,
which `payloads_read` counts. `ArchivePatchOptions::with_verified_payloads` also compares the bytes of every pair the
checksums call equal. `include`, `ignore`, and `exclude_extensions` narrow the comparison; a side holding no entry in
scope is refused, since every entry of the other side would count as a difference. Publication follows the packer's
destination rules above, including `with_force` and rollback.

## Headers and volume limits

Volumes receive a mountable `[header]` by default (`auto_load = true`, `entry_point = $fs_root$\gamedata\`). Keep it
unless you need a different one: the engine treats a headerless `.db` as an encrypted Shadow of Chernobyl archive. A
header that is present must be loadable, or packing refuses it: it needs a `[header]` section, `auto_load`, and an
`entry_point` that is `gamedata` or begins with an alias such as `$fs_root$`, because the engine stops on load
otherwise.

`max_volume_size` caps each finished volume, including chunk headers, header text, stored payloads, and the descriptor
table. Unlike xrCompress's `XRP_TARGET_SIZE`, it cannot be exceeded by the next file or the final table.

The cap defaults to `VOLUME_SIZE_MAX`, matching the engine's `XRP_MAX_SIZE`, and may not go below `VOLUME_SIZE_MIN`.
For an engine fork with a higher limit, call `with_oversized_volumes(true)` before `with_max_volume_size`.
`VOLUME_SIZE_HARD_MAX` still applies.

- A cap too small for the headers, directory rows, and one entry is rejected before writing.
- A file that cannot fit in an empty volume is rejected with its name and required volume size. Raise the cap,
  exclude the file, or store it loose.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-pack
cargo test --locked --doc -p xrf-pack
```

The README is the crate documentation, so its Rust examples are checked as doctests. See the
[packer](src/pack/archive_packer.rs), [unpacker](src/unpack/archive_unpacker.rs),
[world extractor](src/unpack/xray_world_extractor.rs), [patcher](src/patch/archive_patcher.rs), and
[public exports](src/lib.rs).
