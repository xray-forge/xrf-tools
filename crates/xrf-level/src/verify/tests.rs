use std::collections::HashMap;

use uuid::Uuid;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};
use xrf_report::{CheckReport, Status};

use crate::{
  LevelAiFile, LevelAiGraphCensus, LevelAiGraphVerifier, LevelAiGraphVerifyResult, LevelAiHeader, LevelAiNode,
  LevelAiNodes, LevelAiVersion, LevelFile, LevelFileSource, LevelGeometryVerifier, LevelGeometryVerifyResult,
};

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

/// No link, the widest a `CsCop` link holds.
const NONE: u32 = (1 << 23) - 1;

/// A `level.ai` of `links`, one node a cell along x each.
fn ai_map(links: &[[u32; 4]]) -> XrfResult<Vec<u8>> {
  let nodes: Vec<LevelAiNode> = (0u32..)
    .zip(links)
    .map(|(index, links)| LevelAiNode {
      links: *links,
      high_cover: [0; 4],
      low_cover: [0; 4],
      plane: 126,
      packed_xz: index * 4,
      packed_y: 0,
      light: 0,
    })
    .collect();
  let header: LevelAiHeader = LevelAiHeader {
    version: LevelAiVersion::CsCop as u32,
    count: nodes.len() as u32,
    size: 0.7,
    size_y: 1.0,
    bounds: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(7.0, 1.0, 2.1),
    },
    guid: Uuid::nil(),
  };
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelAiFile { header }.write::<XRayByteOrder>(
    &LevelAiNodes::from_nodes::<XRayByteOrder>(LevelAiVersion::CsCop, &nodes)?,
    &mut writer,
  )?;

  Ok(writer.buffer)
}

fn verify_ai(bytes: Option<Vec<u8>>) -> LevelAiGraphVerifyResult {
  let mut level: FakeLevel = FakeLevel::default();

  if let Some(bytes) = bytes {
    level.files.insert(LevelAiFile::FILE_NAME, bytes);
  }

  LevelAiGraphVerifier::verify_source(&level)
}

fn check<'a>(result: &'a LevelAiGraphVerifyResult, id: &str) -> &'a CheckReport {
  result
    .report
    .checks()
    .iter()
    .find(|check| check.id().as_str() == id)
    .expect("the check")
}

/// Nodes 0 and 1 linked both ways along x, 1 linking on to 2 with no link back, and 3 and 4 a part of their own.
#[test]
fn a_one_way_link_and_a_detached_part_are_warnings_with_counts() -> XrfResult {
  let result: LevelAiGraphVerifyResult = verify_ai(Some(ai_map(&[
    [NONE, NONE, 1, NONE],
    [0, NONE, 2, NONE],
    [NONE; 4],
    [NONE, NONE, 4, NONE],
    [3, NONE, NONE, NONE],
  ])?));

  assert_eq!(
    result.census,
    Some(LevelAiGraphCensus {
      nodes: 5,
      links: 5,
      self_links: 0,
      one_way_links: 1,
      one_way_nodes: 1,
      components: 2,
      largest_component: 3,
    })
  );
  assert_eq!(result.report.status(), Status::Passed);

  let graph: &CheckReport = check(&result, "level.ai.graph");

  assert_eq!(graph.status(), Status::Passed);
  assert_eq!(graph.findings().len(), 2);
  assert!(
    graph.findings()[0]
      .message()
      .starts_with("2 nodes lie outside the largest of 2 parts")
  );
  assert!(graph.findings()[0].message().ends_with("3 at (2.1, 0.0, 0.0)"));
  assert!(
    graph.findings()[1]
      .message()
      .ends_with("1 links have no link back, reaching 1 nodes: 2 at (1.4, 0.0, 0.0)")
  );

  Ok(())
}

/// xrAI refuses a map with a node linked to itself.
#[test]
fn a_node_linked_to_itself_fails_the_map() -> XrfResult {
  let result: LevelAiGraphVerifyResult = verify_ai(Some(ai_map(&[[NONE, 0, NONE, NONE]])?));

  assert_eq!(result.report.status(), Status::Failed);
  assert_eq!(check(&result, "level.ai.links").findings().len(), 1);
  assert_eq!(result.census.map(|census| census.self_links), Some(1));

  Ok(())
}

/// A level that ships no AI map is not checked, and an unreadable one fails its read.
#[test]
fn a_missing_map_skips_and_an_unreadable_one_fails() {
  let missing: LevelAiGraphVerifyResult = verify_ai(None);

  assert!(missing.census.is_none());
  assert!(
    missing
      .report
      .checks()
      .iter()
      .all(|check| check.status() == Status::Skipped)
  );

  let unreadable: LevelAiGraphVerifyResult = verify_ai(Some(vec![1, 2, 3]));

  assert_eq!(unreadable.report.status(), Status::Failed);
  assert_eq!(check(&unreadable, "level.ai.read").findings().len(), 1);
}
