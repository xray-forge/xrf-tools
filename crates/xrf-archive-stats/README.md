# xrf-archive-stats

Summarizes what a set of game files holds: counts, sizes, extensions, folders, compression, and which source each file
comes from.

It has two entry points, both returning an `ArchiveStatistics` report:

| Entry point  | Input                                                                                       |
| ------------ | ------------------------------------------------------------------------------------------- |
| `of_volumes` | An open `ArchiveProject` from [xrf-archive](../xrf-archive/README.md): merged archive files |
| `of_world`   | A listing from [xrf-vfs](../xrf-vfs/README.md): loose and archived files, as the game sees  |

A report is computed from metadata the caller already holds. It reads no files and does not validate payloads. Packing
and extraction belong to [xrf-pack](../xrf-pack/README.md).

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

`game/db` is a volume file or a directory of volumes, searched recursively.

- Totals and breakdowns describe the files that win the merge.
- Copies displaced by a later volume (`project.shadowed`) are counted only in `origins.hidden`.
- Names stay as authored. Use a mounted world when the report must follow the engine's path rules and install order.

## Summarize a mounted world

List the files with an `XraySearch`, then pass three of its answers to `of_world`:

| Argument  | Source                        | Contents                                                       |
| --------- | ----------------------------- | -------------------------------------------------------------- |
| `entries` | `list_shadowing_entries`      | One winning file per engine path, with the copies it overrides |
| `mounts`  | `XraySearch::list_roots`      | Mount names; their count becomes `overview.sources`            |
| `sources` | `XraySearch::list_containers` | Loose roots and individual volumes, in search priority order   |

Take all three from the same search. The collector borrows your entries through two traits instead of copying them:

- `ArchiveStatisticsEntry` gives the name and unpacked size. Keep its default `get_size_compressed() -> None`: a world
  listing records no stored sizes.
- `ArchiveWorldStatisticsEntry` adds the winner's container and its hidden copies, each with its own size.
- `of_volumes` needs neither: the crate implements `ArchiveStatisticsEntry` for `ArchiveFileDescriptor` and handles
  directory descriptors itself.

```rust,no_run
use xrf_archive_stats::{ArchiveStatistics, ArchiveStatisticsEntry, ArchiveWorldStatisticsEntry};
use xrf_vfs::{XrayAssetContainer, XrayMountMode, XraySearch, XraySearchPlan, XrayShadowingEntry};

/// A winner and the copies it hides, as the search listed them.
struct WorldEntry(XrayShadowingEntry);

impl ArchiveStatisticsEntry for WorldEntry {
  fn get_name(&self) -> &str {
    self.0.get_logical_path()
  }

  fn get_size_real(&self) -> u64 {
    self.0.get_size()
  }
}

impl ArchiveWorldStatisticsEntry for WorldEntry {
  fn get_container(&self) -> &XrayAssetContainer {
    self.0.get_asset().get_container()
  }

  fn list_shadowed(&self) -> impl Iterator<Item = (&XrayAssetContainer, u64)> {
    self.0.shadowed.iter().map(|copy| (copy.asset.get_container(), copy.size))
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let search: XraySearch = XraySearchPlan::new()
    .with_root_mode("game", "C:\\Games\\Anomaly", XrayMountMode::Installation)?
    .open()?;
  let entries: Vec<WorldEntry> = search.list_shadowing_entries().into_iter().map(WorldEntry).collect();
  let statistics = ArchiveStatistics::of_world(&entries, &search.list_roots(), &search.list_containers());

  println!("{} files from {} mounts", statistics.overview.total.files, statistics.overview.sources);
  println!("{} hidden copies", statistics.origins.hidden.files);

  Ok(())
}
```

Things the collector leaves to you:

- Supply one winner per engine path. It does not resolve conflicts or remove duplicates.
- Supply files only, in the entries and in each hidden-copy list. World origins count every entry, directories
  included.
- A mount can hold several volumes, so `overview.sources` need not equal `origins.sources.len()`.

A single loose `gamedata` directory goes through `of_world` too; its files count under `origins.loose`.

## Read the report

All sizes are `u64` byte counts. `ArchiveMeasure` pairs a file count with its unpacked size (`size_real`).

| Field         | Contents                                                                                  |
| ------------- | ----------------------------------------------------------------------------------------- |
| `overview`    | File totals, source and directory counts, empty files, largest file, mean and median size |
| `extensions`  | One row per extension spelling, largest unpacked total first                              |
| `folders`     | One row per top-level folder, largest unpacked total first; `folder: None` is the root    |
| `sizes`       | Nonempty size bands, smallest first                                                       |
| `largest`     | Up to twenty files, largest first                                                         |
| `compression` | Stored and unpacked totals, and how many files are stored uncompressed                    |
| `volumes`     | Per-volume metadata in merge order; `None` for a world                                    |
| `origins`     | Loose, archived and hidden totals, and what each source contributes                       |

Rows with equal totals are ordered by name.

### Totals

- Extension, folder and size-band rows each sum to `overview.total`. Directories and hidden copies are left out of
  them; `overview.directories` counts directories.
- The mean uses integer division. With an even count, the median is the upper middle size.
- Largest, mean and median are zero when there are no files. Empty files still count and have their own size band.
- A size band includes `from` and excludes `to`; `to: None` is the open-ended last band. The boundaries are
  [`ArchiveSizeBand::FLOORS`](src/report/archive_size_band.rs).
- `largest` is a sample: among files of equal size, which ones make the cut depends on input order.

### Extensions

- An extension keeps its spelling, so `dds` and `DDS` are separate rows.
- `is_declared` says the extension is one [xrf-extension](../xrf-extension/README.md) knows, ignoring case. It does not
  say the payload is valid.
- Names with no extension, a leading dot only (`.gitignore`), or a trailing dot share the `extension: None` row.

### Compression

- `compression` is `None` when no entry supplies a stored size: an empty volume set, or a world. Unavailable is not
  zero; do not substitute unpacked sizes.
- `stored_uncompressed` counts files whose stored and unpacked sizes match, empty files included.
- If a custom entry type mixes known and unknown stored sizes, mixed extension and folder rows report none, and the
  overall total compares the known stored sizes against all unpacked bytes.
- Stored sizes are per entry. They leave out archive headers and tables and can count a shared payload twice, so they
  are not the archive files' size on disk or a forecast of repacking savings.

### Volumes

A volume row copies the volume's own metadata before merging. Its `entries` include directories and files a later
volume overrides, so volume totals need not match the overview. `modified_at` is the file's modification time in Unix
milliseconds, when known.

## Read source contributions

`origins` splits the winning files into `loose` and `archived`, and counts displaced copies in `hidden` at each copy's
own size. Three copies hidden behind one path count three times.

Each `origins.sources` row is a loose root or a single volume:

- `wins` — the source's files that stay visible.
- `hides` — the source's own copies hidden by a higher-priority source.
- `is_loose` — whether its counted copies were loose; `false` for a source with none.

For example, a 1,000-byte loose `configs/system.ltx` overriding a 7,000-byte archived copy gives:

- `overview`: one file, 1,000 bytes;
- the loose source: one `wins` entry;
- the archive: one `hides` entry of 7,000 bytes, also counted in `origins.hidden`.

Source order:

- A volume set lists later volumes first, since they win.
- A world keeps the `sources` order you passed, including sources that contribute nothing. Sources missing from that
  list are appended alphabetically, which says nothing about their priority. Pass each source once, spelled as its
  containers spell it.

The report does not look for unreachable names or collisions within a source; the VFS collision listing does.

## Cost

Two passes over the entries and one sorted size per winning file, for the median. No payload is read.

Report types serialize with camelCase field names. The `typescript-bindings` feature adds Specta metadata for the
frontend bindings.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-archive-stats
cargo doc --locked -p xrf-archive-stats --no-deps
```

Sources: the [report entry points](src/report/archive_statistics.rs), the
[entry traits](src/collect/archive_statistics_entry.rs), the
[breakdown collector](src/collect/archive_statistics_collector.rs), the
[origins collector](src/collect/archive_origins_collector.rs), and the [world tests](src/tests/world.rs).
