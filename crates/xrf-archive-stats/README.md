# xrf-archive-stats

Summarizes file counts, sizes, compression, and source contributions for an open X-Ray archive subject. A **volume
set** is an `ArchiveProject` from [xrf-archive](../xrf-archive/README.md). A **mounted world** is a resolved listing of
loose files and archived entries from [xrf-vfs](../xrf-vfs/README.md), with one winning file per engine path and the
copies it overrides.

Both produce `ArchiveStatistics`. The crate measures existing metadata; it does not open archives, mount directories,
read file payloads, or validate their contents. Use [xrf-pack](../xrf-pack/README.md) to pack or extract files.

## Summarize a volume set

```rust,no_run
use xrf_archive::ArchiveProject;
use xrf_archive_stats::ArchiveStatistics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let project = ArchiveProject::new("game/db")?;
  let statistics = ArchiveStatistics::of_volumes(&project);

  println!("{} files, {} unpacked bytes", statistics.overview.total.files, statistics.overview.total.size_real);

  for row in statistics.extensions.iter().take(5) {
    let extension = row.extension.as_deref().unwrap_or("(no extension)");
    println!("{extension}: {} files, {} bytes", row.measure.files, row.measure.size_real);
  }

  println!("{} overridden copies", statistics.origins.hidden.files);

  Ok(())
}
```

Replace `game/db` with a volume file or a directory containing volumes. Opening can fail; computing the statistics
returns the report directly. File totals and breakdowns describe the merged name table's winners. Displaced copies
retained in `project.shadowed` are counted separately in `origins.hidden`.

## Summarize a mounted world

Call `ArchiveStatistics::of_world(&entries, &mounts, &sources)` with data from the same resolved listing:

| Argument  | Contents                                                                                                           |
| --------- | ------------------------------------------------------------------------------------------------------------------ |
| `entries` | A slice of winning file entries implementing `ArchiveWorldStatisticsEntry`, each retaining the copies it overrides |
| `mounts`  | Mount names, as returned by `XrayProbe::list_roots`; their count becomes `overview.sources`                        |
| `sources` | Loose-root and individual-volume paths in search priority order, as returned by `XrayProbe::list_containers`       |

A mount can hold several volumes, so `mounts` and `sources` need not have the same length. The collector does not
resolve conflicts or deduplicate entries: supply one winner per engine path. A loose gamedata tree uses this same
entry point, with no archived winners or hidden copies unless the supplied listing contains them.

Implement the traits on the entry type your application already retains:

- `ArchiveStatisticsEntry` supplies the name and unpacked byte size. Keep the default `get_size_compressed() -> None`
  for world entries, including archived winners, because the world listing does not record stored sizes.
- `ArchiveWorldStatisticsEntry` adds the winner's `XrayAssetContainer` and an iterator over hidden containers and
  their own unpacked sizes. A hidden copy can differ in size from its winner.

Supply files only to `of_world`: the common breakdown skips entries marked as directories, but the origins collector
counts every supplied world entry. The crate already implements `ArchiveStatisticsEntry` for `ArchiveFileDescriptor`;
`of_volumes` handles directory descriptors itself.

These traits let the collector borrow the existing listing without cloning it into a second entry model. See the
[world tests](src/tests/world.rs) for an implementation over a small entry type.

## Read the report

Sizes are `u64` byte counts. `ArchiveMeasure` pairs a file count with its total unpacked size (`size_real`). Directory
entries are excluded from file totals, size statistics, and breakdowns; `overview.directories` reports their count
separately.

| Field         | Meaning and ordering                                                                                                   |
| ------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `overview`    | Winning file totals, source and directory counts, empty-file count, largest file, mean, and median size                |
| `extensions`  | One row per extension spelling, ordered by unpacked bytes descending, then extension                                   |
| `folders`     | One row per top-level folder, ordered by unpacked bytes descending, then folder                                        |
| `sizes`       | Nonempty size bands in ascending order                                                                                 |
| `largest`     | At most twenty files, ordered by unpacked size descending; equal-size order and cutoff selection depend on input order |
| `compression` | Stored and unpacked totals and the count stored uncompressed, when entries supply stored sizes                         |
| `volumes`     | Per-volume metadata in merge order for a volume set; `None` for a world                                                |
| `origins`     | Winning loose and archived totals, hidden-copy totals, and contributions by source for either subject                  |

The mean uses integer division. For an even file count, the median is the upper middle size, not the average of the
two middle sizes. Largest, mean, and median sizes are zero when there are no files. Zero-length files still count as
files and have their own size band.

Size bands include `from` and exclude `to`; `to: None` marks the open-ended final band. Boundaries are defined by
[`ArchiveSizeBand::FLOORS`](src/report/archive_size_band.rs). Folder grouping stops at the first path segment;
`folder: None` holds files at the root.

### Extension spelling

Extensions come from the last path segment and retain their supplied spelling, so `dds` and `DDS` form separate rows
when both occur. Recognition through [xrf-extension](../xrf-extension/README.md) ignores case: both can have
`is_declared: true`. That flag identifies vocabulary membership, not whether a payload is valid or previewable.

Unknown spellings keep their own rows. Names without an extension, dotfiles such as `.gitignore`, and names ending in
a dot share the `extension: None` row, whose `is_declared` is false. Both slash styles are accepted when splitting
paths; the collector does not otherwise normalize names.

### Compression and volume totals

`of_volumes` measures stored sizes from winning file descriptors. `stored_uncompressed` counts files whose stored
and unpacked sizes match, including zero-byte files. Compression is `None` when no file supplies a stored size, such
as an empty volume set or world entries using the trait's default. An unavailable value is distinct from a measured
zero.

For extension and folder rows, `size_compressed` is present only when every file in that group supplies it. Custom
world entries should keep stored sizes absent consistently: the overall compression collector sums whichever stored
sizes are supplied, while its unpacked total includes every file.

Volume rows copy each volume's metadata before merging: `entries` includes directory entries and files later
overridden by another volume. Their totals therefore need not equal the winning-file overview. `modified_at`, when
available, is the volume file's modification time in Unix milliseconds.

Stored-size totals describe payload sizes per entry. They exclude archive headers and tables and can count shared
payloads more than once; they are not the physical sizes of the archive files or a forecast of repacking savings.

## Interpret source contributions

Both entry points populate `origins`. Its `loose` and `archived` measures divide the winning files by container kind;
`hidden` counts displaced copies separately, using each copy's own size. Several hidden copies of one path count
separately, so `hidden.files` is not the number of paths with overrides.

Each `origins.sources` row names a loose root or an individual volume:

- `wins` measures that source's files which remain visible.
- `hides` measures that source's own copies hidden by higher-priority sources, not files it hides elsewhere.
- `is_loose` is derived from the containers encountered for that source. A source with no counted copies defaults to
  false, so this flag alone cannot classify an empty source.

Volume-set source rows use reverse merge order: later volumes win and appear first. World source rows preserve the
supplied `sources` order, including sources that contribute nothing. Sources encountered in entries but absent from
that list are appended in lexical order; that fallback does not establish their search priority. Supply each source
once, with the same path spelling used by its containers.

The report counts the winners and hidden copies its input retains. It does not independently discover unreachable
names or collisions within a source; use the VFS collision APIs for those findings.

## Cost and checks

The common breakdowns share a pass over winning entries. Origins are collected in a separate pass over winners and
hidden copies. The collector retains and sorts one size per winning file for the median; the largest-files list is
bounded at twenty. No payload bytes are read during either calculation.

Report types implement `Serialize` with camelCase field names. The optional `typescript-bindings` feature adds
frontend type metadata.

From the repository root:

```sh
cargo test --locked -p xrf-archive-stats
cargo doc --locked -p xrf-archive-stats --no-deps
```

See the [report entry points](src/report/archive_statistics.rs), [entry traits](src/collect/archive_statistics_entry.rs),
[breakdown collector](src/collect/archive_statistics_collector.rs), and
[origins collector](src/collect/archive_origins_collector.rs).
