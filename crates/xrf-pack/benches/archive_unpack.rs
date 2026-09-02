//! What a sequential unpack costs per entry, which is where the destination-side work lands.
//!
//! Sequential on purpose. The default pool hides per-entry cost behind however many cores the host has, so a
//! regression in what one entry does is only visible with one worker. Payloads are stored rather than compressed, so
//! what this measures is the filesystem work around an entry rather than the decoder inside it.
//!
//! Depth is the variable because the cost of reaching a destination grows with the number of components on the way
//! down: an entry four deep pays for four steps, and a tree of one directory pays for one.

use std::fs;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use xrf_archive::ArchiveProject;
use xrf_job::ExecutionRequest;
use xrf_pack::{ArchivePackConfig, ArchivePacker, ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

/// Entries per archive, held constant so depth is the only thing that varies.
const ENTRY_COUNT: usize = 512;

/// Directory depths an entry is reached through. One is a flat tree; four is about as deep as a level asset sits.
const DEPTHS: [usize; 2] = [1, 4];

/// Payload bytes per entry, small enough that writing them is not what is being measured.
const PAYLOAD_SIZE: usize = 256;

/// One worker, so per-entry cost is not divided by the host's core count.
fn one_worker() -> NonZeroUsize {
  NonZeroUsize::new(1).expect("one is not zero")
}

/// Write a source tree of `ENTRY_COUNT` stored entries, each `depth` directories below the root.
///
/// `.dds` rather than `.ltx`: the packer compresses configs and stores textures, and a decoder running per entry would
/// drown the filesystem work this measures.
fn create_source_tree(scope: &str, depth: usize) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("bench/{scope}/source"));

  if root.exists() {
    fs::remove_dir_all(&root).expect("a source tree can be replaced");
  }

  let payload: Vec<u8> = vec![0x41; PAYLOAD_SIZE];

  for index in 0..ENTRY_COUNT {
    let mut path: PathBuf = root.clone();

    // Spread over a few directories at each level, so the tree is wide as well as deep and no single directory holds
    // every entry.
    for level in 0..depth {
      path.push(format!("level_{level}_{:02}", index % 8));
    }

    fs::create_dir_all(&path).expect("a source directory can be created");
    fs::write(path.join(format!("entry_{index:04}.dds")), &payload).expect("a source entry can be written");
  }

  root
}

/// Pack a source tree into its own volume set and open it, so the timed region only unpacks.
fn create_project(scope: &str, depth: usize) -> ArchiveProject {
  let source: PathBuf = create_source_tree(scope, depth);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("bench/{scope}/db"));

  if destination.exists() {
    fs::remove_dir_all(&destination).expect("a destination can be replaced");
  }

  let config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "bench");

  ArchivePacker::pack(&config).expect("a synthetic tree packs");

  ArchiveProject::new(&destination).expect("a packed set opens")
}

/// An empty destination for one iteration, outside the timed region.
fn reset_destination(scope: &str) -> PathBuf {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("bench/{scope}/out"));

  if path.exists() {
    fs::remove_dir_all(&path).expect("a destination can be emptied");
  }

  path
}

fn bench_unpack(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("archive_unpack");

  for depth in DEPTHS {
    let scope: String = format!("depth_{depth}");
    let project: ArchiveProject = create_project(&scope, depth);

    // The archive has to hold what the shape asked for, or the figure describes a different tree than its label.
    assert_eq!(
      project.files.values().filter(|file| !file.is_directory).count(),
      ENTRY_COUNT,
      "the synthetic archive holds every entry"
    );

    group.bench_with_input(BenchmarkId::new("sequential", depth), &scope, |bencher, scope| {
      bencher.iter_batched(
        || reset_destination(scope),
        |destination: PathBuf| {
          let result: ArchiveUnpackResult = ExecutionRequest::Workers(one_worker())
            .resolve()
            .install(|| {
              ArchiveUnpacker::unpack_opt(
                &project,
                black_box::<&Path>(&destination),
                ArchiveUnpackOptions::default(),
              )
            })
            .expect("the pool starts")
            .expect("a synthetic archive unpacks");

          // Nothing here is optimized away while the run's own count is what proves it did the work.
          assert_eq!(result.files_unpacked, ENTRY_COUNT, "every entry was written");

          black_box(result)
        },
        BatchSize::PerIteration,
      );
    });
  }

  group.finish();
}

criterion_group!(benches, bench_unpack);
criterion_main!(benches);
