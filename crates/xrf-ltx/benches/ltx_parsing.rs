//! What reading LTX costs, stage by stage.

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::{env, fs};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xrf_ltx::Ltx;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

/// Environment variable naming a directory of real `.ltx` files to parse instead of generated ones.
const CORPUS_VARIABLE: &str = "XRF_BENCH_CORPUS";

/// Section counts to generate, spanning a single config up to a large one.
const SECTION_COUNTS: [usize; 3] = [64, 512, 4096];

/// Fields per generated section, near what weapon and item sections carry.
const FIELDS_PER_SECTION: usize = 12;

/// Builds a config of `sections` sections.
///
/// Shaped like the real thing rather than minimally: an inheritance parent, a comment per section, and values wide
/// enough that the per-token `String` the parser accumulates actually allocates.
fn synthetic_config(sections: usize) -> String {
  let mut text: String = String::new();

  for section in 0..sections {
    text.push_str(&format!("; section {section} generated for the parser benchmark\n"));
    text.push_str(&format!("[sect_{section:05}]:sect_base\n"));

    for field in 0..FIELDS_PER_SECTION {
      text.push_str(&format!("field_{field:02} = value_{section:05}_{field:02}_padding\n"));
    }

    text.push('\n');
  }

  text
}

/// Reads every `.ltx` under a directory, one string per file, or `None` when the directory holds none.
fn corpus_files(root: &Path) -> Option<Vec<String>> {
  let mut files: Vec<String> = Vec::new();
  let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];

  while let Some(directory) = pending.pop() {
    let Ok(entries) = fs::read_dir(&directory) else {
      continue;
    };

    for entry in entries.flatten() {
      let path: PathBuf = entry.path();

      if path.is_dir() {
        pending.push(path);
      } else if path.extension().is_some_and(|extension| extension == "ltx") {
        // Lossy: a corpus carries Windows-1251 configs, and re-encoding them here would measure the encoder.
        if let Ok(bytes) = fs::read(&path) {
          files.push(String::from_utf8_lossy(&bytes).into_owned());
        }
      }
    }
  }

  // Only the files that parse standalone. One that does not is a finding about the corpus rather than about the
  // parser, and timing its failure would report a rate the parser never achieved.
  files.retain(|text| Ltx::read_from_str(text).is_ok());

  (!files.is_empty()).then_some(files)
}

fn bench_parsing(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("ltx_parsing");

  for sections in SECTION_COUNTS {
    let text: String = synthetic_config(sections);

    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_with_input(BenchmarkId::new("synthetic", sections), &text, |bencher, text| {
      bencher.iter(|| black_box(Ltx::read_from_str(black_box(text))));
    });
  }

  // Absent unless a corpus was named, so a run on a machine without an installation is a clean skip rather than a
  // failure — and a figure quoted from this bench always says which of the two modes produced it.
  match env::var(CORPUS_VARIABLE).ok().map(PathBuf::from) {
    Some(root) => match corpus_files(&root) {
      Some(files) => {
        let bytes: usize = files.iter().map(String::len).sum();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("corpus", files.len()), &files, |bencher, files| {
          bencher.iter(|| {
            for text in files {
              black_box(Ltx::read_from_str(black_box(text)).expect("a corpus file to parse"));
            }
          });
        });
      }
      None => println!("{CORPUS_VARIABLE} names {}, which holds no .ltx files", root.display()),
    },
    None => println!("{CORPUS_VARIABLE} is unset, so only the synthetic inputs ran"),
  }

  group.finish();
}

/// Builds a config whose sections form `sections` chains of four, so inheritance has real work to flatten.
///
/// The parser's own generator declares a parent it never defines, which parses but cannot resolve. This one closes the
/// chains, and stacks four deep because a resolved section copies every ancestor's fields in turn.
fn synthetic_inheritance_config(sections: usize) -> String {
  let mut text: String = String::new();

  for section in 0..sections {
    text.push_str(&format!("[base_{section:05}]\n"));

    for field in 0..FIELDS_PER_SECTION {
      text.push_str(&format!("field_{field:02} = value_{section:05}_{field:02}_padding\n"));
    }

    for generation in 1..4 {
      text.push_str(&format!(
        "\n[child_{section:05}_{generation}]:{}\n",
        if generation == 1 {
          format!("base_{section:05}")
        } else {
          format!("child_{section:05}_{}", generation - 1)
        }
      ));
      text.push_str(&format!("own_{generation} = value_{section:05}_{generation}\n"));
    }

    text.push('\n');
  }

  text
}

/// Writes a tree of one entry point over `files` wildcard-included section files, and answers the entry point.
fn synthetic_tree(files: usize) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ltx_bench_tree/{files}"));
  let sections: PathBuf = root.join("sections");

  // Cleared because the include is a wildcard: a file an earlier run left behind would join this run's matches and
  // quietly change what the figure describes.
  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&sections).expect("sections directory");

  for file in 0..files {
    let mut text: String = String::new();

    text.push_str(&format!("[file_{file:04}_base]\n"));

    for field in 0..FIELDS_PER_SECTION {
      text.push_str(&format!("field_{field:02} = value_{file:04}_{field:02}_padding\n"));
    }

    text.push_str(&format!("\n[file_{file:04}_child]:file_{file:04}_base\nown = {file}\n"));

    fs::write(sections.join(format!("section_{file:04}.ltx")), text).expect("section file");
  }

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"sections\\section_*.ltx\"\n").expect("entry point");

  entry
}

/// What the stages after and around the parser cost.
fn bench_pipeline(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("ltx_pipeline");

  for sections in SECTION_COUNTS {
    let text: String = synthetic_config(sections);

    // Reformatting is a second parse that builds no `Ltx`, so a command that verifies and formats parses twice.
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_with_input(BenchmarkId::new("format", sections), &text, |bencher, text| {
      bencher.iter(|| black_box(Ltx::format_from_str(black_box(text))));
    });
  }

  for sections in SECTION_COUNTS {
    let text: String = synthetic_inheritance_config(sections);
    let parsed: Ltx = Ltx::read_from_str(&text).expect("the generated chains to parse");

    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_with_input(BenchmarkId::new("inherit", sections), &parsed, |bencher, parsed| {
      // Cloned per iteration because flattening consumes the document; the clone is inside the measurement, so this
      // figure is an upper bound on the pass rather than the pass alone.
      bencher.iter(|| black_box(black_box(parsed).clone().into_inherited()));
    });
  }

  // Whole-tree reads, which is what a project pass does per entry point: discover includes, read and decode each file,
  // merge them, then flatten inheritance.
  for files in [16, 128] {
    let entry: PathBuf = synthetic_tree(files);

    group.throughput(Throughput::Elements(files as u64));
    group.bench_with_input(BenchmarkId::new("resolve_tree", files), &entry, |bencher, entry| {
      bencher.iter(|| black_box(Ltx::read_from_file_standard(black_box(entry))));
    });
  }

  group.finish();
}

criterion_group!(benches, bench_parsing, bench_pipeline);
criterion_main!(benches);
