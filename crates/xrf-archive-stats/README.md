# xrf-archive-stats

Breaks an open archive subject down the ways a person asks about it: by extension, by folder, by size, by volume, and
by where a mounted world's files actually come from.

## Break down a subject

```rust,no_run
use xrf_archive::ArchiveProject;
use xrf_archive_stats::ArchiveStatistics;

fn main() -> xrf_error::XrfResult {
  let project = ArchiveProject::new("game/db")?;
  let statistics = ArchiveStatistics::of_volumes(&project);

  println!("{} files, {} bytes", statistics.overview.total.files, statistics.overview.total.size_real);

  for row in statistics.extensions.iter().take(5) {
    println!("{:?}: {} files", row.extension, row.measure.files);
  }

  Ok(())
}
```

A mounted world uses `ArchiveStatistics::of_world(entries, mounts)` instead, where `entries` are the winning entries
each carrying the sized copies it hides.

## Two subjects, one pass

Every section falls out of a single walk of the entries. Asking for them one at a time would mean walking an
installation-sized listing once per section, so there is one entry point per subject and no per-section call.

The crate sits above [xrf-archive](../xrf-archive/README.md) and [xrf-vfs](../xrf-vfs/README.md) rather than inside
either, the way [xrf-pack](../xrf-pack/README.md) does. The same question is asked of a volume set, whose name table
records a stored size per entry, and of a mounted world, which answers loose files and archived entries through one
lookup and records no stored size at all. Putting the shared arithmetic in either crate would give one of them
knowledge of the other's subject.

## Supply entries without copying them

Entries arrive through `ArchiveStatisticsEntry` - a name, an unpacked size, an optional stored size, and whether the
entry names a directory. It is a trait rather than a shape this crate owns because a mounted world's listing is tens of
thousands of entries and neither caller can afford a second copy of it to be measured.

`ArchiveWorldStatisticsEntry` adds what only a world can answer: where the winning copy sits, and the sized copies it
hides. A volume set does not implement it, because it cannot answer either question honestly.

The crate implements the base trait for `ArchiveFileDescriptor`. A consumer implements it for whatever it already
retains.

## Absent is not empty

What only one kind of subject can answer is `Option`, never an empty collection.

A volume set has no `origins`: it merges its name table on the way in, keeping one entry per name, so the copies it
folded away are gone before anything could count them. A world has no `compression` and no `volumes`: a loose file has
no stored size, so any compression figure over a mixed tree would be invented.

An empty answer would claim the section looked and found nothing, which is a different claim from the subject being
unable to see it.

## Extensions are reported as found

Rows are keyed by the spelling on disk, annotated with whether [xrf-extension](../xrf-extension/README.md) declares it.
That vocabulary is a closed list of spellings evidence has been seen for, so folding what it does not declare into an
`Other` row would discard the evidence that grows it — an Anomaly volume set holds nine undeclared spellings, including
`som` files, which is a real X-Ray format the list is missing. Files with no extension get their own row, where nothing
to recognize is distinguished from something unrecognized.

## Limitations and checks

Breakdowns are ordered heaviest first with the key breaking ties, so two readings of one subject agree. Folders are
grouped at depth one only. The largest-files section is bounded at twenty entries and kept as the walk runs
rather than by sorting every name afterwards. Size bands are powers of two with zero-length files kept apart, so a band
means the same thing in every subject.

Nothing here reads a payload: every figure comes from a name table or a mounted listing that was already produced.
Compression is what the format recorded, not what a re-compression would achieve.

Run `cargo test --locked -p xrf-archive-stats` from the repository root. The optional `typescript-bindings` feature enables
frontend type metadata.

See the [report and its entry points](src/report/archive_statistics.rs), the
[collector](src/collect/archive_statistics_collector.rs), and the
[origins collector](src/collect/archive_origins_collector.rs) only a world needs. Size bands are the report type's own
scale: [`ArchiveSizeBand::FLOORS`](src/report/archive_size_band.rs).
