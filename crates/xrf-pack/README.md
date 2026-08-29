# xrf-pack

Packing and unpacking X-Ray `.db` archive volumes.

The volume format itself lives in `xrf-archive`; this crate is the two directions between a volume set and a directory
on disk. `ArchivePacker` builds volumes from a directory tree, `ArchiveUnpacker` writes them back out to one, and
`ArchivePackConfig` describes a pack as an LTX file — which is why this crate, not `xrf-archive`, holds the `xrf-ltx`
dependency.

```rust,no_run
use xrf_archive::ArchiveProject;
use xrf_pack::{ArchivePackConfig, ArchivePacker, ArchiveUnpacker};

# fn main() -> xrf_error::XrfResult {
// Pack a directory into volumes; an LTX pack config refines the defaults.
let config: ArchivePackConfig = ArchivePackConfig::new("C:\\work\\gamedata", "C:\\work\\db", "my_mod")
  .with_ltx_file("C:\\work\\pack.ltx")?;
let packed = ArchivePacker::pack(&config)?;

// Unpack a volume set back into a directory. Unpacking is synchronous and builds a pool of the given size, so a
// caller on an async executor runs the whole call on a blocking thread. One worker is a sequential run.
let project: ArchiveProject = ArchiveProject::new("C:\\Games\\Anomaly\\db")?;
let unpacked = ArchiveUnpacker::unpack(&project, "C:\\work\\unpacked", ArchiveUnpacker::default_concurrency())?;

println!("packed {} files; unpacked {} volumes", packed.files_total, unpacked.archives.len());
# Ok(())
# }
```

Volumes are written with a mountable `[header]` by default: a headerless archive not named `xdb` is assumed by the
engine to be an encrypted Shadow of Chernobyl archive and decrypts into nonsense, so the harmless case is the default.
Volume size is capped at the engine's `XRP_MAX_SIZE`.
