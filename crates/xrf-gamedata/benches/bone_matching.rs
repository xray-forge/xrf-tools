//! What the mesh check's bone comparison costs per mesh-and-motion pair.
//!
//! `MeshAssetsVerifier` compares an OGF skeleton against an OMF's bones with
//! `bones.iter().any(|it| !omf_bones.contains(&it.name.as_str()))`, which is O(mesh × motion) in string comparisons.
//! It runs only when the two counts already match, so the matching case is the worst case: `any` returns false only
//! after every bone has been searched for.
//!
//! The question this answers is not whether a `HashSet` is faster — it is — but whether the difference is worth a
//! change at the sizes real skeletons have. Multiply the per-pair figure by the pairs a sweep performs before
//! concluding anything: Anomaly's whole `meshes` check costs 1,216ms.
//!
//! Names are generated rather than read from a corpus. The shape that matters here is the count and the shared prefix
//! that forces comparisons past the first byte, and both are parameters.

use std::collections::HashSet;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Bone counts spanning what X-Ray skeletons actually carry.
const BONE_COUNTS: [usize; 4] = [32, 64, 128, 256];

/// Builds a skeleton's bone names.
fn bone_names(count: usize) -> Vec<String> {
  (0..count).map(|index| format!("bip01_spine_bone_{index:03}")).collect()
}

/// The current call site: a linear scan of the motion's bones for each of the mesh's.
fn matches_by_scan(mesh: &[String], motion: &[&str]) -> bool {
  !mesh.iter().any(|name| !motion.contains(&name.as_str()))
}

/// The proposed shape: the motion's bones hashed once, then one lookup per mesh bone.
fn matches_by_set(mesh: &[String], motion: &[&str]) -> bool {
  let motion: HashSet<&str> = motion.iter().copied().collect();

  !mesh.iter().any(|name| !motion.contains(name.as_str()))
}

fn bench_bone_matching(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("bone_matching");

  for count in BONE_COUNTS {
    let mesh: Vec<String> = bone_names(count);
    let motion: Vec<&str> = mesh.iter().map(String::as_str).collect();

    group.bench_with_input(BenchmarkId::new("linear_scan", count), &count, |bencher, _| {
      bencher.iter(|| black_box(matches_by_scan(black_box(&mesh), black_box(&motion))));
    });

    group.bench_with_input(BenchmarkId::new("hash_set", count), &count, |bencher, _| {
      bencher.iter(|| black_box(matches_by_set(black_box(&mesh), black_box(&motion))));
    });
  }

  group.finish();
}

criterion_group!(benches, bench_bone_matching);
criterion_main!(benches);
