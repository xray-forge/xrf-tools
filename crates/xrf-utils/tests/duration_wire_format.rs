//! Every serialized duration in the workspace is a millisecond count.
//!
//! `serde` has no way to say "encode all `Duration` fields like this", so each one carries
//! `#[serde(with = "xrf_utils::duration_ms")]` itself - and forgetting it fails silently, by falling
//! back to serde's own `{ secs, nanos }` object beside fields that emit a number. That is exactly how
//! `xrf_report::CheckReport` came to answer the same question two ways inside one report.
//!
//! This walks the workspace sources rather than any one type, so a report type added later is
//! covered the day it is written instead of the day someone reads its output.

use std::fs;
use std::path::{Path, PathBuf};

/// A field declaration of a duration type, which is what needs the attribute.
fn get_duration_field_name(line: &str) -> Option<&str> {
  let declaration: &str = line.trim();
  let declaration: &str = declaration.strip_prefix("pub ").unwrap_or(declaration);
  let (name, rest) = declaration.split_once(": ")?;

  if !name.chars().all(|char| char.is_ascii_alphanumeric() || char == '_') || name.is_empty() {
    return None;
  }

  matches!(rest, "Duration," | "Option<Duration>,").then_some(name)
}

fn collect_sources(directory: &Path, sources: &mut Vec<PathBuf>) {
  let Ok(entries) = fs::read_dir(directory) else {
    return;
  };

  for entry in entries.flatten() {
    let path: PathBuf = entry.path();

    if path.is_dir() {
      // Build output is generated, and `node_modules` is not ours to audit.
      if !matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("target" | "node_modules")
      ) {
        collect_sources(&path, sources);
      }
    } else if path.extension().is_some_and(|extension| extension == "rs") {
      sources.push(path);
    }
  }
}

#[test]
fn every_serialized_duration_is_a_millisecond_count() {
  let workspace: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .and_then(Path::parent)
    .expect("the utility crate to sit inside the workspace")
    .to_path_buf();

  let mut sources: Vec<PathBuf> = Vec::new();

  collect_sources(&workspace, &mut sources);
  sources.sort();

  assert!(!sources.is_empty(), "the workspace should hold Rust sources to audit");

  let mut unannotated: Vec<String> = Vec::new();

  for source in &sources {
    let Ok(contents) = fs::read_to_string(source) else {
      continue;
    };

    let lines: Vec<&str> = contents.lines().collect();
    let mut is_serialized: bool = false;

    for (index, line) in lines.iter().enumerate() {
      if line.contains("derive(") && line.contains("Serialize") {
        is_serialized = true;
      } else if line.starts_with('}') {
        is_serialized = false;
      }

      if !is_serialized {
        continue;
      }

      let Some(field) = get_duration_field_name(line) else {
        continue;
      };

      // Attributes and doc comments stack above a field, and these carry a specta one too.
      let attributes: String = lines[index.saturating_sub(6)..index].join("\n");

      if !attributes.contains("duration_ms") {
        unannotated.push(format!(
          "{}:{}  {field}",
          source
            .strip_prefix(&workspace)
            .unwrap_or(source)
            .display()
            .to_string()
            .replace('\\', "/"),
          index + 1
        ));
      }
    }
  }

  assert!(
    unannotated.is_empty(),
    "these serialized durations would emit a `{{ secs, nanos }}` object instead of a millisecond \
     count. Add `#[serde(with = \"xrf_utils::duration_ms\")]`, or \
     `#[serde(with = \"xrf_utils::optional_duration_ms\")]` for an `Option`:\n  {}",
    unannotated.join("\n  ")
  );
}
