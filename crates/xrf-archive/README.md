# xrf-archive

The X-Ray `.db`/`.xdb` archive volume format: headers, entry descriptors, and moving payloads in and out.

This crate answers how a volume set is encoded, and nothing above that: `xrf-vfs` mounts one as an asset source, and
`xrf-pack` packs and unpacks them. Neither could own the format without the other reaching into it, so it lives
below both. Prefer those crates unless you are working on the format itself — resolution, shadowing, and logical paths
all live in `xrf-vfs`.

`ArchiveProject` is the entry point: it merges a volume set into a single name table with the later volume winning,
matching how the engine registers archives. Volumes merge in path order, which is the name-sorted depth-first order
`CLocatorAPI::Recurse` walks a directory in; no directory is special to it. Precedence *between* archive directories is
their `fsgame.ltx` declaration order, which `xrf-vfs` applies when it plans an installation.

```rust,no_run
use xrf_archive::ArchiveProject;

# fn main() -> xrf_error::XrfResult {
// One volume, or every volume under a directory.
let project: ArchiveProject = ArchiveProject::new("C:\\Games\\Anomaly\\db")?;
let bytes: Vec<u8> = project.read_file_bytes("configs\\system.ltx")?;

println!("{} entries, {} bytes unpacked", project.files.len(), project.get_real_size());
# Ok(())
# }
```

Entry names are read as Windows-1251, like every engine text format. Corrupt volumes are errors, never panics: a
declared size is checked against the volume before it can reach an allocation, so a bad `.db` becomes a skipped or
reported mount rather than an abort.

`write_descriptor_contents` streams one archived entry into an open file, decompressing and CRC-checking on the way —
the primitive `xrf-pack`'s unpacker and single-file extraction share so they cannot drift.
