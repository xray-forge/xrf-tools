use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::{EnvFlagKind, EnvFlags};

#[test]
fn names_every_variable_once() {
  let mut seen: HashSet<&str> = HashSet::new();

  for flag in EnvFlags::ALL {
    assert!(seen.insert(flag.name), "{} is listed twice", flag.name);
  }
}

#[test]
fn names_the_workspaces_own_by_area() {
  for flag in EnvFlags::ALL.iter().filter(|flag| flag.name.starts_with("XRF_")) {
    let area: &str = flag.name.split('_').nth(1).unwrap_or_default();

    assert!(
      ["RENDER", "APP", "CLI", "BUILD", "DLSS"].contains(&area),
      "{} names no area as XRF_<AREA>_<NAME> does",
      flag.name
    );
  }
}

#[test]
fn reports_only_what_a_running_binary_reads() {
  let names: Vec<&str> = EnvFlags::list_readings().iter().map(|it| it.name).collect();

  assert!(names.contains(&"XRF_RENDER_BACKEND"));
  assert!(names.contains(&"RUST_LOG"));
  assert!(!names.contains(&"XRF_BUILD_KIND"));
  assert!(!names.contains(&"XRF_DLSS_RUNTIME"));
  assert!(
    EnvFlags::list_read_by("xrf-cli")
      .iter()
      .all(|flag| flag.kind != EnvFlagKind::Build)
  );
}

/// `ENVIRONMENT.md` is what the table writes: a stale one is rewritten and fails this once.
#[test]
fn keeps_the_document_up_to_date() {
  let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ENVIRONMENT.md");
  let expected: String = EnvFlags::to_markdown();
  let current: String = fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");

  if current != expected {
    fs::write(&path, &expected).expect("ENVIRONMENT.md is writable");

    panic!("ENVIRONMENT.md was stale and is rewritten: commit it");
  }
}
