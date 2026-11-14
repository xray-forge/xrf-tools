use xrf_error::{XrfError, XrfResult};
use xrf_level::BuildPrjFile;
use xrf_math::{Aabb, EPS_L, Vector3d};

use crate::geometry::geometry_bounds::GeometryBounds;
use crate::geometry::geometry_models::GeometryModels;
use crate::geometry::geometry_node::GeometryNode;
use crate::geometry::geometry_node_kind::GeometryNodeKind;
use crate::geometry::geometry_placement::GeometryPlacement;
use crate::geometry::geometry_sector::GeometrySector;
use crate::geometry::geometry_visuals::GeometryVisuals;

/// A level's visual tree and its sectors, as xrLC lays them out (`g_tree`; `CBuild::BuildSectors`,
/// `xrLC/xrSectors.cpp:7`).
///
/// The tree starts with the static visuals in order, then each multi-use reference's trees followed by its LOD node
/// where it has one. Each visual's sector is its material's, a tree's and a LOD's their reference's; the sectors are
/// numbered up to the highest, and each must hold a visual.
///
/// Each sector then grows its hierarchy (`CSector::BuildHierrarhy`, `xrLC/Sector.cpp:40`): the size starts at 8 m and
/// doubles while within twice the scene's widest side. At each size, each of the sector's nodes not yet under another
/// starts a node and takes in, one at a time, the node whose box joined with the node's, grown by `EPS_L`, stays within
/// the size and is smallest by volume (the first of equals); a node that took in nothing is undone, and one that did
/// joins the tree with its bounds taken from all the geometry under it. The sector's root is the last node it grew, or
/// its one node where it grew none. A portal is listed in both its sectors.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryHierarchy {
  pub nodes: Vec<GeometryNode>,
  pub sectors: Vec<GeometrySector>,
}

impl GeometryHierarchy {
  /// The smallest size a hierarchy node starts from, `c_SS_maxsize / 4`.
  pub const FIRST_SIZE: f32 = 8.0;

  /// The tree of `visuals` and `models`, with `project`'s portals.
  ///
  /// # Errors
  ///
  /// Returns an error for a sector holding no visual, a portal naming a sector past the last or with the same sector
  /// on both sides, which xrLC asserts on.
  pub fn build(visuals: &GeometryVisuals, models: &GeometryModels, project: &BuildPrjFile) -> XrfResult<Self> {
    let mut points: Vec<Vec<Vector3d>> = Vec::new();
    let mut nodes: Vec<GeometryNode> = Vec::new();

    for (index, visual) in visuals.visuals.iter().enumerate() {
      let geometry: Vec<Vector3d> = visual.vertices.iter().map(|vertex| vertex.position).collect();

      nodes.push(Self::node(
        GeometryNodeKind::Visual(index as u32),
        visual.sector,
        &geometry,
      ));
      points.push(geometry);
    }

    let mut lods = models.lods.iter().enumerate().peekable();

    for (index, tree) in models.trees.iter().enumerate() {
      let visual = &models.models[tree.model as usize].visuals[tree.visual as usize];
      let geometry: Vec<Vector3d> = visual
        .vertices
        .iter()
        .map(|vertex| GeometryPlacement::transform(&tree.transform, &vertex.position))
        .collect();

      nodes.push(Self::node(GeometryNodeKind::Tree(index as u32), tree.sector, &geometry));
      points.push(geometry);

      let is_last_of_reference: bool = models
        .trees
        .get(index + 1)
        .is_none_or(|next| next.reference != tree.reference);

      if is_last_of_reference && let Some((lod_index, lod)) = lods.next_if(|(_, lod)| lod.reference == tree.reference) {
        let first: usize = nodes.len() - lod.trees.len();
        let geometry: Vec<Vector3d> = points[first..].concat();

        nodes.push(Self::node(
          GeometryNodeKind::Lod(lod_index as u32),
          lod.sector,
          &geometry,
        ));
        points.push(geometry);
      }
    }

    let sector_count: usize = nodes
      .iter()
      .map(|node| usize::from(node.sector))
      .max()
      .map_or(0, |last| last + 1);
    // A LOD took its trees in when it was made (`OGF_Node::AddChield`).
    let mut is_under_lod: Vec<bool> = vec![false; models.trees.len()];

    for tree in models.lods.iter().flat_map(|lod| &lod.trees) {
      is_under_lod[*tree as usize] = true;
    }

    let mut is_connected: Vec<bool> = nodes
      .iter()
      .map(|node| matches!(node.kind, GeometryNodeKind::Tree(tree) if is_under_lod[tree as usize]))
      .collect();
    let mut sectors: Vec<GeometrySector> = Vec::with_capacity(sector_count);

    for sector in 0..sector_count {
      let root: u32 = Self::grow(&mut nodes, &mut points, &mut is_connected, sector as u16)?;

      sectors.push(GeometrySector {
        root,
        portals: Vec::new(),
      });
    }

    for (index, portal) in project.portals.iter().enumerate() {
      for side in [portal.sector_front, portal.sector_back] {
        sectors
          .get_mut(usize::from(side))
          .ok_or_else(|| XrfError::new_invalid_error(format!("Portal {index} faces sector {side} of {sector_count}")))?
          .portals
          .push(index as u16);
      }
    }

    for (index, sector) in sectors.iter_mut().enumerate() {
      sector.portals.sort_unstable();

      if sector.portals.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(XrfError::new_invalid_error(format!(
          "Sector {index} is on both sides of a portal"
        )));
      }
    }

    Ok(Self { nodes, sectors })
  }

  /// One sector's hierarchy, and its root.
  fn grow(
    nodes: &mut Vec<GeometryNode>,
    points: &mut Vec<Vec<Vector3d>>,
    is_connected: &mut Vec<bool>,
    sector: u16,
  ) -> XrfResult<u32> {
    let scene: Aabb = nodes
      .iter()
      .map(|node| node.bounds.bounds)
      .reduce(|first, second| Self::merge(&first, &second))
      .unwrap_or(Aabb {
        min: Vector3d::ZERO,
        max: Vector3d::ZERO,
      });
    let grow = |value: f32, by: f32| value + by;
    let scene_size: Vector3d = Vector3d::new(
      grow(scene.max.x, EPS_L) - grow(scene.min.x, -EPS_L),
      grow(scene.max.y, EPS_L) - grow(scene.min.y, -EPS_L),
      grow(scene.max.z, EPS_L) - grow(scene.min.z, -EPS_L),
    );
    let mut delimiter: f32 = scene_size.x.max(scene_size.y.max(scene_size.z)) * 2.0;
    let mut limit: f32 = Self::FIRST_SIZE;
    let mut root: Option<u32> = None;

    if delimiter <= limit {
      delimiter *= 2.0;
    }

    while limit <= delimiter {
      let size: usize = nodes.len();
      let members: Vec<usize> = (0..size).filter(|index| nodes[*index].sector == sector).collect();

      for start in &members {
        if is_connected[*start] {
          continue;
        }

        let mut children: Vec<u32> = vec![*start as u32];
        let mut bounds: Aabb = nodes[*start].bounds.bounds;

        is_connected[*start] = true;

        loop {
          let mut best: Option<(usize, f32)> = None;

          for candidate in &members {
            if is_connected[*candidate] {
              continue;
            }

            let merged: Aabb = Self::merge(&bounds, &nodes[*candidate].bounds.bounds);
            let side: Vector3d = &merged.max - &merged.min;

            if [side.x, side.y, side.z].iter().any(|value| value + EPS_L > limit) {
              continue;
            }

            let volume: f32 = side.x * side.y * side.z;

            if best.is_none_or(|(_, smallest)| volume < smallest) {
              best = Some((*candidate, volume));
            }
          }

          let Some((chosen, _)) = best else {
            break;
          };

          children.push(chosen as u32);
          is_connected[chosen] = true;
          bounds = Self::merge(&bounds, &nodes[chosen].bounds.bounds);
        }

        if children.len() > 1 {
          let geometry: Vec<Vector3d> = children
            .iter()
            .flat_map(|child| points[*child as usize].clone())
            .collect();

          nodes.push(Self::node(GeometryNodeKind::Hierarchy(children), sector, &geometry));
          points.push(geometry);
          is_connected.push(false);
          root = Some((nodes.len() - 1) as u32);
        } else {
          is_connected[*start] = false;
        }
      }

      limit *= 2.0;
    }

    match root {
      Some(root) => Ok(root),
      None => (0..nodes.len())
        .find(|index| nodes[*index].sector == sector && !is_connected[*index])
        .map(|index| index as u32)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Sector {sector} holds no visual"))),
    }
  }

  /// A node of `kind` in `sector`, its bounds from `geometry`.
  fn node(kind: GeometryNodeKind, sector: u16, geometry: &[Vector3d]) -> GeometryNode {
    GeometryNode {
      kind,
      sector,
      bounds: GeometryBounds::of(geometry),
    }
  }

  /// `Fbox::merge`.
  fn merge(first: &Aabb, second: &Aabb) -> Aabb {
    Aabb {
      min: Vector3d::new(
        first.min.x.min(second.min.x),
        first.min.y.min(second.min.y),
        first.min.z.min(second.min.z),
      ),
      max: Vector3d::new(
        first.max.x.max(second.max.x),
        first.max.y.max(second.max.y),
        first.max.z.max(second.max.z),
      ),
    }
  }
}
