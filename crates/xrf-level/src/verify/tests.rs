use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_report::Status;

use crate::{LevelFile, LevelFileSource, LevelGeometryVerifier, LevelGeometryVerifyResult};

/// A level's files held in memory, by name.
#[derive(Default)]
struct FakeLevel {
  files: HashMap<&'static str, Vec<u8>>,
}

impl LevelFileSource for FakeLevel {
  fn read(&self, name: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(self.files.get(name).cloned())
  }

  fn describe(&self, name: &str) -> String {
    format!("fake/{name}")
  }
}

fn verify(level: &FakeLevel) -> LevelGeometryVerifyResult {
  LevelGeometryVerifier::new(level).run()
}

/// A path naming no level must not pass every check by having nothing to check.
#[test]
fn a_level_without_its_bundle_fails_on_one_read_finding() {
  let result: LevelGeometryVerifyResult = verify(&FakeLevel::default());
  let read = &result.report.checks()[0];

  assert_eq!(result.report.status(), Status::Failed);
  assert_eq!(read.id().as_str(), "level.read");
  assert_eq!(read.findings().len(), 1);
  assert_eq!(read.findings()[0].subject(), Some("fake/level"));
  assert!(read.findings()[0].message().contains("not found"));
}

/// A bundle that is no level is one defect, reported once rather than once per reading of it.
#[test]
fn an_unreadable_bundle_is_one_read_finding() {
  let level: FakeLevel = FakeLevel {
    files: HashMap::from([(LevelFile::FILE_NAME, vec![1, 2, 3])]),
  };
  let result: LevelGeometryVerifyResult = verify(&level);

  assert_eq!(result.report.status(), Status::Failed);
  assert_eq!(result.report.checks()[0].findings().len(), 1);
  assert_eq!(result.report.count_findings(), 1);
}
