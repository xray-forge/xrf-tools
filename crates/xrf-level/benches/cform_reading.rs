//! What reading and writing a level's collision form costs: a million vertices and twice as many faces on a large level.

#![expect(clippy::print_stdout, reason = "the bench says on the console which inputs it ran")]

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::{env, fs};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xrf_chunk::ChunkWriter;
use xrf_level::{LevelCformFace, LevelCformFile, LevelCformGeometry, LevelCformHeader};
use xrf_math::Vector3d;

/// Environment variable naming a directory searched for `level.cform` files to read instead of generated ones.
const CORPUS_VARIABLE: &str = "XRF_BENCH_CORPUS";

/// Grid sides to generate: a small level, and one near Zaton's 1.1 M vertices and 1.9 M faces.
const GRID_SIDES: [u32; 2] = [256, 1024];

/// A flat grid of `side` × `side` vertices and two faces a cell, framed as the file the engine reads.
fn grid_form(side: u32) -> Vec<u8> {
  let vertices: Vec<Vector3d<f32>> = (0..side * side)
    .map(|index| Vector3d::new((index % side) as f32, 0.0, (index / side) as f32))
    .collect();
  let face = |vertices: [u32; 3], index: u32| LevelCformFace {
    vertices,
    material: (index % 40) as u16,
    is_shadow_suppressed: index.is_multiple_of(3),
    is_wallmark_suppressed: index.is_multiple_of(5),
    sector: (index % 300) as u16,
  };
  let faces: Vec<LevelCformFace> = (0..(side - 1) * (side - 1))
    .flat_map(|cell| {
      let corner: u32 = cell / (side - 1) * side + cell % (side - 1);

      [
        face([corner, corner + side, corner + 1], cell * 2),
        face([corner + 1, corner + side, corner + side + 1], cell * 2 + 1),
      ]
    })
    .collect();
  let geometry: LevelCformGeometry = LevelCformGeometry::new(vertices, faces).expect("a grid naming its own vertices");
  let file: LevelCformFile = LevelCformFile {
    header: LevelCformHeader::of(&geometry).expect("a grid header"),
  };
  let mut writer: ChunkWriter = ChunkWriter::new();

  file
    .write_with_geometry(&geometry, &mut writer)
    .expect("a written grid");

  writer.buffer
}

/// Every `level.cform` under a directory, as bytes.
fn corpus_forms(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
  let mut forms: Vec<(PathBuf, Vec<u8>)> = Vec::new();
  let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];

  while let Some(directory) = pending.pop() {
    let Ok(entries) = fs::read_dir(&directory) else {
      continue;
    };

    for entry in entries.flatten() {
      let path: PathBuf = entry.path();

      if path.is_dir() {
        pending.push(path);
      } else if path.file_name().is_some_and(|name| name == LevelCformFile::FILE_NAME)
        && let Ok(bytes) = fs::read(&path)
      {
        forms.push((path, bytes));
      }
    }
  }

  forms.sort_by(|first, second| first.0.cmp(&second.0));
  forms
}

/// Reads a form's header and geometry, and answers its face count so the read cannot be skipped.
fn read_faces(bytes: Vec<u8>) -> usize {
  let (_, geometry): (LevelCformFile, LevelCformGeometry) =
    LevelCformFile::read_with_geometry_from_bytes(bytes).expect("a readable collision form");

  geometry.get_faces().len()
}

fn bench_form(criterion: &mut Criterion, name: &str, bytes: &[u8]) {
  let (file, geometry): (LevelCformFile, LevelCformGeometry) =
    LevelCformFile::read_with_geometry_from_bytes(bytes.to_vec()).expect("a readable collision form");
  let mut written: ChunkWriter = ChunkWriter::new();

  file
    .write_with_geometry(&geometry, &mut written)
    .expect("a written collision form");
  assert_eq!(written.buffer, bytes, "{name} to come back byte for byte");

  let mut group = criterion.benchmark_group("cform");

  group.throughput(Throughput::Elements(u64::from(file.header.face_count)));
  // The bytes are cloned outside the measurement, since the read takes them by value as a level open does.
  group.bench_function(BenchmarkId::new("read", name), |bencher| {
    bencher.iter_batched(
      || bytes.to_vec(),
      |bytes| black_box(read_faces(black_box(bytes))),
      BatchSize::LargeInput,
    );
  });
  group.bench_function(BenchmarkId::new("write", name), |bencher| {
    bencher.iter(|| {
      let mut writer: ChunkWriter = ChunkWriter::new();

      file
        .write_with_geometry(black_box(&geometry), &mut writer)
        .expect("a written collision form");
      black_box(writer.buffer)
    });
  });
  group.finish();
}

fn bench_cform(criterion: &mut Criterion) {
  for side in GRID_SIDES {
    bench_form(criterion, &format!("grid_{side}"), &grid_form(side));
  }

  // Absent unless a corpus was named, so a figure quoted from this bench always says which input produced it.
  match env::var(CORPUS_VARIABLE).ok().map(PathBuf::from) {
    Some(root) => {
      let forms: Vec<(PathBuf, Vec<u8>)> = corpus_forms(&root);

      if forms.is_empty() {
        println!("{CORPUS_VARIABLE} names {}, which holds no level.cform", root.display());
      }

      for (path, bytes) in forms {
        let level: String = path
          .parent()
          .and_then(Path::file_name)
          .map_or_else(|| String::from("level"), |name| name.to_string_lossy().into_owned());

        bench_form(criterion, &level, &bytes);
      }
    }
    None => println!("{CORPUS_VARIABLE} is unset, so only the generated forms ran"),
  }
}

criterion_group!(benches, bench_cform);
criterion_main!(benches);
