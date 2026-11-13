use std::time::{Duration, Instant};

use xrf_chunk::XRayByteOrder;
use xrf_math::Vector3d;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

use crate::verify::level_ai_graph_census::LevelAiGraphCensus;
use crate::verify::level_ai_graph_verify_result::LevelAiGraphVerifyResult;
use crate::verify::level_file_source::LevelFileSource;
use crate::{LevelAiFile, LevelAiHeader, LevelAiNodes};

/// Checks a level's AI map as a graph, as xrAI's `-verify` does (`xrAI/verify_level_graph.cpp`): a node linked to
/// itself fails the map, as it fails GSC's ("AI map is CORRUPTED"); links with no link back and parts of the map no link
/// reaches are warnings with their counts and first positions, since GSC shipped maps with both. Reachability is the
/// map's connected parts, a link joining either way, computed once rather than flooded from every one-way node.
pub struct LevelAiGraphVerifier;

impl LevelAiGraphVerifier {
  /// How many positions a finding lists before it stops.
  const FIRST_POSITIONS: usize = 5;

  /// Reads `level.ai` from `source` and checks it; every check is skipped for a level that ships no AI map, as a
  /// multiplayer level may.
  pub fn verify_source<S: LevelFileSource + ?Sized>(source: &S) -> LevelAiGraphVerifyResult {
    let started: Instant = Instant::now();
    let subject: String = source.describe(LevelAiFile::FILE_NAME);
    let read = source.read(LevelAiFile::FILE_NAME).and_then(|bytes| {
      bytes
        .map(LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>)
        .transpose()
    });

    match read {
      Ok(Some((file, nodes))) => Self::verify(&file.header, &nodes, &subject),
      Ok(None) => Self::new_result(None, started.elapsed(), [GraphCheck::SKIPPED; 3]),
      Err(error) => Self::new_result(
        None,
        started.elapsed(),
        [
          GraphCheck::of(vec![Self::new_finding("level.ai.read", &subject, error.to_string())]),
          GraphCheck::SKIPPED,
          GraphCheck::SKIPPED,
        ],
      ),
    }
  }

  /// Checks the map of `header` and `nodes`, its findings naming `subject`. A link is a neighbour when it names one of
  /// the nodes held, as the engine's `valid_vertex_id` takes it against the header's count.
  pub fn verify(header: &LevelAiHeader, nodes: &LevelAiNodes, subject: &str) -> LevelAiGraphVerifyResult {
    let started: Instant = Instant::now();
    let count: u32 = header.count.min(u32::try_from(nodes.get_count()).unwrap_or(u32::MAX));
    let links: Vec<[u32; 4]> = nodes
      .iter_nodes::<XRayByteOrder>()
      .take(count as usize)
      .map(|node| node.links)
      .collect();
    let mut census: LevelAiGraphCensus = LevelAiGraphCensus {
      nodes: count,
      ..LevelAiGraphCensus::default()
    };
    let mut self_linked: Vec<u32> = Vec::new();
    let mut is_one_way_node: Vec<bool> = vec![false; links.len()];
    let mut parts: Vec<u32> = (0..count).collect();
    let mut sizes: Vec<u32> = vec![1; links.len()];

    for (node, node_links) in (0u32..).zip(&links) {
      for (side, link) in node_links.iter().enumerate() {
        if *link >= count {
          continue;
        }

        census.links += 1;

        if *link == node {
          census.self_links += 1;

          if self_linked.last() != Some(&node) {
            self_linked.push(node);
          }

          continue;
        }

        // `single_link`: the neighbour's opposite edge does not lead back here.
        if links[*link as usize][(side + 2) % 4] != node {
          census.one_way_links += 1;
          is_one_way_node[*link as usize] = true;
        }

        Self::join(&mut parts, &mut sizes, node, *link);
      }
    }

    census.one_way_nodes = is_one_way_node.iter().filter(|is_one_way| **is_one_way).count() as u32;

    let roots: Vec<u32> = (0..count).map(|node| Self::find(&mut parts, node)).collect();
    let largest: Option<u32> = (0..count)
      .filter(|node| roots[*node as usize] == *node)
      .max_by_key(|root| (sizes[*root as usize], std::cmp::Reverse(*root)));

    census.components = (0..count).filter(|node| roots[*node as usize] == *node).count() as u32;
    census.largest_component = largest.map_or(0, |root| sizes[root as usize]);

    let position = |node: u32| -> String {
      nodes
        .get_node::<XRayByteOrder>(node as usize)
        .map(|decoded| {
          let Vector3d { x, y, z } = header.get_node_position(&decoded);

          format!("{node} at ({x:.1}, {y:.1}, {z:.1})")
        })
        .unwrap_or_default()
    };
    let first = |found: &mut dyn Iterator<Item = u32>| -> String {
      found
        .take(Self::FIRST_POSITIONS)
        .map(position)
        .collect::<Vec<String>>()
        .join(", ")
    };
    let mut link_findings: Vec<Finding> = Vec::new();
    let mut graph_findings: Vec<Finding> = Vec::new();

    if !self_linked.is_empty() {
      link_findings.push(Self::new_finding(
        "level.ai.self-link",
        subject,
        format!(
          "{} links name the node they leave, which xrAI refuses: nodes {}",
          census.self_links,
          first(&mut self_linked.iter().copied())
        ),
      ));
    }

    if census.one_way_nodes > 0 {
      graph_findings.push(Self::new_finding(
        "level.ai.one-way",
        subject,
        format!(
          "{} links have no link back, reaching {} nodes: {}",
          census.one_way_links,
          census.one_way_nodes,
          first(&mut (0..count).filter(|node| is_one_way_node[*node as usize]))
        ),
      ));
    }

    if let Some(largest) = largest.filter(|_| census.components > 1) {
      graph_findings.push(Self::new_finding(
        "level.ai.disconnected",
        subject,
        format!(
          "{} nodes lie outside the largest of {} parts no link joins; a node of each other part: {}",
          count - census.largest_component,
          census.components,
          first(&mut (0..count).filter(|node| roots[*node as usize] == *node && *node != largest))
        ),
      ));
    }

    Self::new_result(
      Some(census),
      started.elapsed(),
      [
        GraphCheck::of(Vec::new()),
        GraphCheck::of(link_findings),
        GraphCheck {
          status: Status::Passed,
          findings: graph_findings,
        },
      ],
    )
  }

  /// The part `node` belongs to, halving each path it walks.
  fn find(parts: &mut [u32], mut node: u32) -> u32 {
    while parts[node as usize] != node {
      parts[node as usize] = parts[parts[node as usize] as usize];
      node = parts[node as usize];
    }

    node
  }

  /// Joins the parts of `a` and `b`, the smaller under the larger.
  fn join(parts: &mut [u32], sizes: &mut [u32], a: u32, b: u32) {
    let (a, b) = (Self::find(parts, a), Self::find(parts, b));

    if a == b {
      return;
    }

    let (larger, smaller) = if sizes[a as usize] >= sizes[b as usize] {
      (a, b)
    } else {
      (b, a)
    };

    parts[smaller as usize] = larger;
    sizes[larger as usize] += sizes[smaller as usize];
  }

  fn new_finding(rule: &'static str, subject: &str, message: String) -> Finding {
    Finding::new(RuleId::of_static(rule), Some(subject.to_owned()), message)
  }

  /// The result of the three checks, `level.ai.read`, `level.ai.links` and `level.ai.graph`, in that order.
  fn new_result(
    census: Option<LevelAiGraphCensus>,
    duration: Duration,
    checks: [GraphCheck; 3],
  ) -> LevelAiGraphVerifyResult {
    let checks: Vec<CheckReport> = ["level.ai.read", "level.ai.links", "level.ai.graph"]
      .into_iter()
      .zip(checks)
      .map(|(id, check)| CheckReport::new(CheckId::of_static(id), check.status, Some(duration), check.findings))
      .collect();

    LevelAiGraphVerifyResult {
      census,
      duration,
      report: Report::new(checks),
    }
  }
}

/// One check's verdict and what it found, before it is named.
struct GraphCheck {
  status: Status,
  findings: Vec<Finding>,
}

impl GraphCheck {
  const SKIPPED: Self = Self {
    status: Status::Skipped,
    findings: Vec::new(),
  };

  /// A check failing on any finding.
  fn of(findings: Vec<Finding>) -> Self {
    Self {
      status: Status::from_is_valid(findings.is_empty()),
      findings,
    }
  }
}
