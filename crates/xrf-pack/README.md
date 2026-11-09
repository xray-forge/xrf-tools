# xrf-pack

Packs, unpacks and patches X-Ray `.db` archive volumes.

- Reading the volume format is [xrf-archive](../xrf-archive/README.md)'s job; resolving engine paths across loose files
  and archives is [xrf-vfs](../xrf-vfs/README.md)'s.
- This crate writes: volumes from a directory tree, files out of a volume set or a mounted search, and patch volumes
  holding what one tree changed against another.

| Operation            | Does                                            |
| -------------------- | ----------------------------------------------- |
| `ArchivePacker`      | Builds volumes from a directory                 |
| `ArchiveUnpacker`    | Extracts a volume set into a directory          |
| `XrayWorldExtractor` | Extracts the winning assets of a mounted search |
| `ArchivePatcher`     | Compares two trees and writes patch volumes     |

Each is a stateless unit struct with a plain entry point and an `_opt` one taking run options (progress, cancellation
and the like). What to act on is always an argument.

## Pack and unpack

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

`ArchivePackConfig` holds the packing settings.

- `with_config_file` loads selection rules and a header from `.ltx` (the xrCompress dialect) or `.json`;
  `write_config_to_path` saves them. The extension picks the format; any other is refused. `ArchivePackConfigJson`
  defines the JSON fields.
- Paths, volume name and run options stay in the calling code.

### Running

- Operations are synchronous. On an async executor, run them on a blocking thread.
- `pack_opt`, `unpack_opt` and `extract_directory_opt` take options built by `with_job`, for progress and cancellation.
- A cancelled run returns `Ok` with `outcome` set to `JobOutcome::Cancelled`. Check `outcome` before treating a result
  as complete.

### Packing rules

**Names.**

- Archive names are the source-relative paths with `\` separators, in their authored case.
- Two files whose names fold to one engine name (possible on a case-sensitive host) are refused before any volume is
  written, all such names reported at once. One file reached under two spellings is packed once.
- A host path that is not valid Unicode is refused, not dropped.

**Selection.**

- The include, exclude and skip rules decide what is packed; `files_skipped` counts what they left out.
- `is_with_skip_list` (on by default) applies xrCompress's built-in skip list, widened from `.db` to every extension the
  engine mounts as a volume (`.db*` and `.xdb*`, as `ArchiveDescriptor::is_volume_path` tests).
- A run that selects no files is refused.

**Destination.**

- A destination equal to or inside the source is refused, since the next run would pack this run's volumes. Write
  volumes beside the source, such as `db` next to `gamedata`. A source inside the destination is fine.
- A destination already holding volumes of the same name is refused unless `ArchivePackOptions::with_force` allows it;
  `ArchivePacker::list_published_volumes` checks before starting.
- An unforced run starts with none of its volumes present, so a failure or cancellation removes every volume it wrote.
- A forced run cannot roll back or remove leftover volumes from a longer earlier set;
  `ArchivePackResult::volumes_opened` lists what it left on disk.

**Compression and naming.**

- In the default `ArchivePackMode::Compress`, only `ltx`, `xml` and `script` files are LZO-compressed, as xrCompress
  does, and only when it saves enough. `Store` stores everything.
- A file whose payload equals one already in the same volume costs only a descriptor row (`files_aliased`).
- A set of one volume is named `<name>.db`; a larger one `<name>.db0`, `<name>.db1`, and so on (`.xdb` with
  `ArchiveVolumeExtension::Xdb`).

### Unpacking and extraction

**What is written.**

- `unpack` writes every entry at its volume's unpack root (`[header] entry_point`, such as `gamedata\`) joined with its
  authored name.
- `extract_directory` writes the entries under one name prefix, matched on whole path components without case, relative
  to that prefix. An empty match is a `NotFound` error.
- `extract_file` writes to the exact path the caller supplies, which may be a link.
- `XrayWorldExtractor::extract_directory` extracts the winning assets of an `XraySearch`, loose overrides included.
  Paths below the prefix are kept; an empty prefix extracts everything. Shadowed copies are left out, and directories
  are created only to hold files.

**Safety.**

- One file is written per engine path. Names differing only in case are one file to the engine, which reads the copy
  with the greater `ArchiveFileDescriptor::get_precedence` (the later volume); that copy is written, on every host.
- Every entry name is checked before the first write. A name with a `.` or `..` component or a colon fails the run.
- `unpack` and `extract_directory` follow existing symlinks, junctions and reparse points under the destination. For an
  untrusted destination, `ArchiveExtractOptions::with_link_policy(HostLinkPolicy::Refuse)` (`xrf_utils::HostLinkPolicy`) refuses them.
- Extraction can leave completed files behind after a failure or cancellation.

## Build a patch

`ArchivePatcher` compares a base (`input`) with what the patch should deliver (`target`).

- `compare` reports changes without writing.
- `patch` also writes added and modified entries into new volumes.
- A patch cannot remove base entries, so files only in the base are not reported.

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

**Sides.**

- Without a `target`, the patcher splits one installation: its archives against its loose overrides, from a single
  reading of `fsgame.ltx`. An input with no archive volumes is refused.
- `with_target` compares against a separate tree instead.

**Equality.**

- Two copies are equal when size and CRC32 match. An archive records its CRC; a loose file is read to compute one
  (`payloads_read` counts these).
- `ArchivePatchOptions::with_verified_payloads` also compares the bytes of every pair the checksums call equal.
- `include`, `ignore` and `exclude_extensions` narrow the comparison. A side with no entry in scope is refused, since
  every entry on the other side would count as a difference.

**Publishing.** The packer's destination rules apply, including `with_force` and rollback. A destination inside either
side's root is refused: write patches elsewhere, then deploy them where the installation loads them after its base
archives.

## Headers and volume size

**Header.** Volumes get a mountable `[header]` by default (`auto_load = true`, `entry_point = $fs_root$\gamedata\`).
Keep it unless you need another: the engine treats a headerless `.db` as an encrypted Shadow of Chernobyl archive.

A header you supply must load, or packing refuses it. It needs a `[header]` section, `auto_load`, and an `entry_point`
that is `gamedata` or an alias followed by a path, such as `$fs_root$\gamedata\`
(`ArchiveMetadata::is_mountable_entry_point`); otherwise the engine stops on load or reads past the value.

**Size cap.** `max_volume_size` caps each finished volume, counting chunk headers, header text, payloads and the
descriptor table. Unlike xrCompress's `XRP_TARGET_SIZE`, the next file or the final table cannot push past it.

- It defaults to `ArchivePackConfig::VOLUME_SIZE_MAX`, the engine's `XRP_MAX_SIZE`, and may not go below
  `VOLUME_SIZE_MIN`.
- For an engine fork with a higher limit, call `with_oversized_volumes(true)` before `with_max_volume_size`.
  `VOLUME_SIZE_HARD_MAX` still applies.
- A cap too small for the headers, directory rows and one entry is rejected before writing.
- A file that cannot fit in an empty volume is rejected with its name and the volume size it needs. Raise the cap,
  exclude the file, or ship it loose.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-pack
cargo test --locked --doc -p xrf-pack
```

Sources: the [packer](src/pack/archive_packer.rs), [unpacker](src/unpack/archive_unpacker.rs),
[world extractor](src/unpack/xray_world_extractor.rs), [patcher](src/patch/archive_patcher.rs), and
[public exports](src/lib.rs).
