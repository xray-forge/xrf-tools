# xrf-archive

Reads X-Ray `.db`/`.xdb` archive headers, entry descriptors, and payloads.

Use [xrf-vfs](../xrf-vfs/README.md) for logical paths and mounted asset lookup, or
[xrf-pack](../xrf-pack/README.md) to pack and extract files.

## Read a volume set

`ArchiveProject` is the entry point: it merges a volume set into a single name table with the later volume winning,
matching how the engine registers archives. Volumes merge in path order. Precedence between archive directories is
their `fsgame.ltx` declaration order, applied by `xrf-vfs` when it plans an installation.

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

Entry names use Windows-1251. Readers check declared sizes against volume bounds and return errors for malformed data.
`new` discovers volumes recursively; `new_shallow` restricts discovery to the selected directory.

`write_descriptor_contents` streams one entry into an open file, decompressing and checking its CRC.
`list_shared_payloads` groups descriptors that address the same payload; it does not identify which entry was packed first.

See the [project API](src/project/archive_project.rs) and [exports](src/lib.rs).
