# xrf-pack

Packing and unpacking X-Ray `.db` archive volumes.

The volume format itself lives in `xrf-archive`; this crate is the two directions between a volume set and a directory
on disk. `ArchivePacker` builds volumes from a directory tree, `ArchiveUnpacker` writes them back out to one, and
`ArchivePackConfig` describes a pack, and reads or writes the file-owned half of itself as either `ltx` or `json`
(`ArchivePackConfigJson`) — which is why this crate, not `xrf-archive`, holds the `xrf-ltx` dependency.

```rust,no_run
use xrf_archive::ArchiveProject;
use xrf_pack::{ArchivePackConfig, ArchivePacker, ArchiveUnpacker};

# fn main() -> xrf_error::XrfResult {
// Pack a directory into volumes; a configuration file refines the defaults. The codec is chosen from the extension,
// so naming `pack.json` reads the same selection rules out of json instead.
let config: ArchivePackConfig = ArchivePackConfig::new("C:\\work\\gamedata", "C:\\work\\db", "my_mod")
  .with_config_file("C:\\work\\pack.ltx")?;
let packed = ArchivePacker::pack(&config)?;

// Unpack a volume set back into a directory. Unpacking is synchronous and builds a pool of its own, so a caller on an
// async executor runs the whole call on a blocking thread. `unpack_opt` bounds the pool, reports progress, and makes
// the run cancellable; one worker is a sequential run.
let project: ArchiveProject = ArchiveProject::new("C:\\Games\\Anomaly\\db")?;
let unpacked = ArchiveUnpacker::unpack(&project, "C:\\work\\unpacked")?;

println!("packed {} files; unpacked {} volumes", packed.files_total, unpacked.archives.len());
# Ok(())
# }
```

Bulk extraction is contained. `unpack` and `extract_directory` lay out archive-controlled names, so both write through a
rooted walk that creates one component at a time and refuses an existing symlink, junction, or other reparse point below
the destination: an entry that spells no traversal still cannot escape through a link the destination already held.
`extract_file` is intentionally different, because a caller naming one exact output path may name a linked one.

Volumes are written with a mountable `[header]` by default: a headerless archive not named `xdb` is assumed by the
engine to be an encrypted Shadow of Chernobyl archive and decrypts into nonsense, so the harmless case is the default.

`max_volume_size` is a hard maximum on each finished volume file and is itself capped at the engine's `XRP_MAX_SIZE`.
Every byte counts against it: the header chunk, the data chunk, each payload as it is actually stored, and the
descriptor chunk appended at the end. This is stricter than the `XRP_TARGET_SIZE` of xrCompress, which only tests the
position reached before the next file and is therefore overshot by that file and by everything written after it.

A cap the packer cannot keep is refused rather than exceeded, with no oversized-volume exception:

- A cap with no room for one volume's chunk headers, header text, and directory rows plus a single entry is rejected
  before anything is written.
- An entry whose payload and descriptor row do not fit an otherwise empty volume is rejected, naming the file and the
  volume size it would have needed. Raise the cap, drop the file, or store it loose.
