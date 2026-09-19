# xrf-archive

Opens X-Ray archive volumes, lists their entries, and reads or copies their unpacked contents. A **volume** is one file,
such as `configs.db0` or `patch.xdb`. A **volume set** is a group of volumes merged into one name table through
`ArchiveProject`.

Use [xrf-vfs](../xrf-vfs/README.md) to resolve assets across loose files and archives with installation-level
precedence. Use [xrf-pack](../xrf-pack/README.md) to create archives or extract files and directories to disk.

## Open a volume set and read a file

```rust,no_run
use xrf_archive::ArchiveProject;

fn main() -> xrf_error::XrfResult {
  let project = ArchiveProject::new("game/db")?;
  let bytes = project.read_file_bytes(r"configs\system.ltx")?;

  println!("{} volumes, {} entries", project.archives.len(), project.files.len());
  println!("Read {} unpacked bytes", bytes.len());

  Ok(())
}
```

Opening reads the volume headers and name tables. File payloads are read on demand, so a successful open does not
prove that every file can be decompressed.

| Method                                   | Input and discovery                                                        |
| ---------------------------------------- | -------------------------------------------------------------------------- |
| `ArchiveProject::new(path)`              | One file, or archive volumes recursively beneath a directory               |
| `ArchiveProject::new_shallow(path)`      | One file, or archive volumes directly inside a directory                   |
| `ArchiveProject::discover_volumes(path)` | The paths `new` would open, in merge order, without parsing their contents |

Directory discovery recognizes extensions beginning with `db` or `xdb`, ignoring case, including `.db0` and `.xdb1`.
A directly named file is opened as a volume regardless of its extension. Opening fails when no volume is found, a
volume cannot be parsed, or the directory walk fails. Discovery alone may return an empty list.

### Names and precedence

Volumes merge in path order. When two volumes contain the same entry name, the later volume wins. A directory named
`patches` has no special priority here. To apply the archive-directory declaration order from `fsgame.ltx`, open the
installation through `xrf-vfs`.

Entry names are decoded from Windows-1251 and kept as authored. Reads by name use an exact lookup: they do not fold
case, replace separators, or add a volume's unpack root. Use a name from `project.files` when its spelling is unknown;
use `xrf-vfs` when the input is an engine logical path that needs normalization and resolution.

## Inspect entries and volumes

The project exposes three collections:

| Collection | Contents                                                                                      |
| ---------- | --------------------------------------------------------------------------------------------- |
| `files`    | The merged name table, including file and directory entries; iteration order is unspecified   |
| `archives` | Volume descriptors in merge order, including paths, timestamps, unpack roots, and size totals |
| `shadowed` | File entries displaced by later volumes; displaced directory entries are excluded             |

Each `ArchiveFileDescriptor` carries its name, stored and unpacked sizes, CRC32, payload offset, and volume index.
`project.get_volume_of(entry)` returns the corresponding `ArchiveDescriptor`. Keep descriptors with the project that
owns them: a volume index refers to a position in that project's `archives` collection.

Use `entry.is_directory` to distinguish directory rows from files. Directory names end in a separator; a zero-byte
entry without one is an empty file. Thus `project.files.len()` counts entries, including directories.

`ArchiveDescriptor::output_root_path` comes from the volume metadata's `[header] entry_point`, with the leading alias
removed. It describes an unpack root, not an additional prefix to pass to `read_file_bytes`.

### Understand size totals

`project.get_real_size()` sums unpacked sizes in the merged table, and `get_compressed_size()` sums their stored sizes.
The same methods on an `ArchiveDescriptor` report that volume's totals before merging across volumes.

These are sums per entry. They exclude archive headers and tables, and shared payloads can be counted more than once.
They therefore do not report the physical size of the archive files or the number of distinct stored bytes.

`project.list_shared_payloads()` finds groups of two or more file entries with the same volume, offset, stored size,
unpacked size, and CRC. It excludes directories and shadowed entries. Groups are ordered by volume and offset, with
names sorted within each group. This identifies shared storage; it does not reveal which entry a packer wrote first.

## Read bytes or text

`read_file_bytes(name)` returns the complete unpacked payload as a `Vec<u8>`. It applies no preview size limit or
extension filter. A caller choosing this method must decide how large a file it is willing to hold in memory.

For display text, use `read_file_as_string(name)`. It checks `project.read_policy` before reading the payload and
returns `ArchiveReadResult` with the name, Windows-1251-decoded content, and unpacked byte size. It does not detect UTF-8
or parse formats such as LTX or XML.

```rust,no_run
use xrf_archive::ArchiveProject;

fn main() -> xrf_error::XrfResult {
  let mut project = ArchiveProject::new("game/db")?;
  project.read_policy.maximum_size = 2 * 1024 * 1024;

  let text = project.read_file_as_string(r"configs\system.ltx")?;
  println!("{} ({} bytes)\n{}", text.name, text.size, text.content);

  Ok(())
}
```

The default text limit is 10 MiB. Allowed extensions include LTX, XML, INI, JSON, scripts, and shader sources; extension
checks ignore case. `ArchiveReadPolicy::supports_file` checks only the extension, while `require_text_read` also
checks the unpacked size. A size exactly equal to the limit is allowed.

The policy also carries texture, image, audio, format-description, and chunk-tree preview limits for consumers.
Those fields do not automatically constrain byte reads or copies; consumers must apply the relevant limits themselves.
See the [policy](src/project/archive_read_policy.rs) and [default limits and extensions](src/project/constants.rs).

## Reuse open volumes for multiple reads

For a batch, call `project.open_volumes()` once. The returned `ArchiveOpenVolumes` borrows the project and holds one
file handle per volume until it is dropped. `read_file_bytes` opens this set for each call; retaining it avoids
reopening the volumes for every entry.

```rust,no_run
use xrf_archive::ArchiveProject;

fn main() -> xrf_error::XrfResult {
  let project = ArchiveProject::new("game/db")?;
  let volumes = project.open_volumes()?;

  for entry in project.files.values() {
    if entry.is_directory || entry.size_real > 1024 {
      continue;
    }

    let bytes = volumes.read_bytes(entry)?;
    println!("{}: {} unpacked bytes", entry.name, bytes.len());
  }

  Ok(())
}
```

`volumes.read_bytes(entry)` can also read a descriptor from `project.shadowed` when an older copy is needed.
`volumes.get_unpack_root_of(entry)` returns its volume's unpack root.

To copy one entry into an already opened file, use `volumes.write_contents(&mut target, entry)`. Position the target
at offset zero before calling: the method writes at its current cursor and sets its final length to the unpacked size.
Stored entries are copied through a bounded buffer; compressed entries are decompressed in memory before writing.
This method copies a payload out of an archive. Archive creation and extraction-path handling belong to `xrf-pack`.
A failed copy can leave a partially written target.

## Validation and checks

Header reads check chunk bounds and descriptor row sizes. Payload reads and copies check that the stored byte range
fits its volume. When stored and unpacked sizes differ, the payload is LZO-decompressed, and both the resulting size
and CRC32 are checked. When the sizes are equal, the payload is treated as uncompressed and its CRC is not checked.

Missing entry names, invalid volume indexes, out-of-bounds payloads, decompression failures, and failed size or CRC
checks return `XrfError`. File operations also propagate I/O failures. The crate validates archive structure and
payload decoding; interpreting a mesh, texture, configuration, or other unpacked asset belongs to its format crate.

From the repository root:

```sh
cargo test --locked -p xrf-archive
cargo doc --locked -p xrf-archive --no-deps
```

The README is included in the crate documentation, so its Rust examples are checked as doctests. See the
[project operations](src/project/archive_project.rs), [payload reads and copies](src/project/archive_open_volumes.rs),
and [public exports](src/lib.rs).
