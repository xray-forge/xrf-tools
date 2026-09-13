# xrf-pack

Packs, unpacks, and patches X-Ray `.db` archive volumes. The volume format is defined in
[xrf-archive](../xrf-archive/README.md).

## Pack and unpack

`ArchivePacker` builds volumes from a directory; `ArchiveUnpacker` extracts them into a directory.
`ArchivePackConfig` holds the packing settings. Use `with_config_file` to load selection rules and a header from
`.ltx` or `.json`, or `write_config_to_path` to save them. The extension selects the format; `ArchivePackConfigJson`
defines the JSON fields. Paths, volume name, and run options remain in the calling code.

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

Operations are synchronous. On an async executor, run the call on a blocking thread. Use `unpack_opt` with
`ArchiveUnpackOptions::with_job` for progress and cancellation.

`unpack` and `extract_directory` reject existing symlinks, junctions, and other reparse points below the destination
root. `extract_file` writes to the exact path supplied by the caller, which may be linked.

`XrayWorldExtractor::extract_directory` extracts the winning assets from an `XrayProbe`, including loose overrides.
It preserves paths below the selected prefix; an empty prefix extracts the whole world. Shadowed copies are omitted.

Packing refuses existing volumes with the same name unless forced. Forced packing cannot roll back or remove stale
volumes from a longer previous set. Extraction can leave completed files behind after failure or cancellation.

## Build a patch

`ArchivePatcher` compares what a patch is built against with what it delivers. `compare` reports changes without
writing; `patch` also writes added and modified entries into new volumes. A patch cannot remove base entries, so
files present only in the base are not reported.

With one installation as input, the patcher compares its archives against its loose overrides using `fsgame.ltx`.
Set `target` to compare against a separate tree. Write patches outside the input and deploy them where the
installation loads them after its base archives.

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

## Headers and volume limits

Volumes receive a mountable `[header]` by default. Keep it unless you need a different header: the engine treats a
headerless `.db` as an encrypted Shadow of Chernobyl archive.

`max_volume_size` caps each finished volume, including chunk headers, header text, stored payloads, and the descriptor
table. Unlike xrCompress's `XRP_TARGET_SIZE`, it cannot be exceeded by the next file or the final table.

The cap defaults to `VOLUME_SIZE_MAX`, matching the engine's `XRP_MAX_SIZE`. For an engine fork with a higher limit,
call `with_oversized_volumes(true)` before `with_max_volume_size`. `VOLUME_SIZE_HARD_MAX` still applies.

- A cap too small for the headers, directory rows, and one entry is rejected before writing.
- A file that cannot fit in an empty volume is rejected with its name and required volume size. Raise the cap,
  exclude the file, or store it loose.
