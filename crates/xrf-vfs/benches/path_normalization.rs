//! What normalizing a logical path costs, on both of the two paths through it.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use xrf_vfs::XrayLogicalPath;

/// Component depths spanning what real logical paths reach.
///
/// A config sits two deep, a level mesh four or five. Ten is past the top of the range, so the cost's growth with
/// length is visible rather than inferred from one point.
const DEPTHS: [usize; 3] = [2, 5, 10];

/// A path already in the form `normalize` would produce: lower case, backslashes, no leading or trailing separator.
fn canonical_path(depth: usize) -> String {
  (0..depth)
    .map(|index| format!("segment_{index:02}"))
    .collect::<Vec<_>>()
    .join("\\")
}

/// The same path in the form a caller hands over: forward slashes and mixed case, so every branch of the rewrite runs.
fn rewritable_path(depth: usize) -> String {
  (0..depth)
    .map(|index| format!("Segment_{index:02}"))
    .collect::<Vec<_>>()
    .join("/")
}

fn bench_normalization(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("path_normalization");

  for depth in DEPTHS {
    let canonical: String = canonical_path(depth);
    let rewritable: String = rewritable_path(depth);

    group.bench_with_input(BenchmarkId::new("canonical", depth), &canonical, |bencher, path| {
      bencher.iter(|| black_box(XrayLogicalPath::normalize(black_box(path)).expect("a valid logical path")));
    });

    group.bench_with_input(BenchmarkId::new("rewritten", depth), &rewritable, |bencher, path| {
      bencher.iter(|| black_box(XrayLogicalPath::normalize(black_box(path)).expect("a valid logical path")));
    });
  }

  group.finish();
}

criterion_group!(benches, bench_normalization);
criterion_main!(benches);
