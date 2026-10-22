use xrf_math::Vector3d;
use xrf_spawn::{AlifeObject, AlifeObjectInherited, Patrol, Shape, SpawnLevelGraph};
use xrf_visual::VisualTransform;

use crate::plugins::levels::state::LevelSpawn;
use crate::plugins::levels::viewer::annotations::level_annotation_kind::LevelAnnotationKind;

/// `RestrictionSpace::eDefaultRestrictorTypeIn` and `eRestrictorTypeIn`: a restrictor keeping the actor inside it.
const RESTRICTOR_TYPES_IN: [u8; 2] = [2, 4];

/// A point seen from above, engine `x` and `z`.
pub type AnnotationPoint = [f32; 2];

/// What a volume's shape covers seen from above.
#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationOutline {
  Circle {
    center: AnnotationPoint,
    radius: f32,
  },
  /// A box's corners from above, their convex hull counterclockwise.
  Polygon(Vec<AnnotationPoint>),
}

impl AnnotationOutline {
  /// Its middle: a circle's centre, a polygon's mean corner.
  pub fn get_center(&self) -> AnnotationPoint {
    match self {
      Self::Circle { center, .. } => *center,
      Self::Polygon(points) => {
        let count: f32 = points.len().max(1) as f32;
        let (x, z): (f32, f32) = points
          .iter()
          .fold((0.0, 0.0), |(x, z), point| (x + point[0], z + point[1]));

        [x / count, z / count]
      }
    }
  }
}

/// One logic volume of the level seen from above: a smart terrain, restrictor, anomaly or level changer.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationVolume {
  pub kind: LevelAnnotationKind,
  pub name: String,
  pub outlines: Vec<AnnotationOutline>,
  /// Whether it keeps the actor inside it, as a restrictor of the in type does.
  pub is_in: bool,
  /// A level changer's destination: the level's name, and the way toward it from this level across the world the
  /// levels share, engine `x` and `z`, where the graph places both.
  pub destination: Option<(String, Option<AnnotationPoint>)>,
}

/// A patrol path from above: its points, and the links between them by index.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationPatrol {
  pub name: String,
  pub points: Vec<AnnotationPoint>,
  pub links: Vec<(usize, usize)>,
}

/// Everything a level map's annotations can draw of a level, engine space seen from above.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelAnnotations {
  pub volumes: Vec<AnnotationVolume>,
  pub patrols: Vec<AnnotationPatrol>,
  /// The game graph's vertices on the level, by id.
  pub graph_points: Vec<(u16, AnnotationPoint)>,
}

impl LevelAnnotations {
  /// What `spawn` and `graph` hold of the level named `level`.
  pub fn of(spawn: &LevelSpawn, graph: &SpawnLevelGraph, level: &str) -> Self {
    Self {
      volumes: spawn
        .objects
        .iter()
        .filter_map(|object| to_volume(object, graph, level))
        .collect(),
      patrols: graph.patrols.iter().map(to_patrol).collect(),
      graph_points: graph.points.iter().map(|(id, point)| (*id, to_point(point))).collect(),
    }
  }
}

/// A point from above.
fn to_point(point: &Vector3d) -> AnnotationPoint {
  [point.x, point.z]
}

/// An object's logic volume from above, or `None` for an object that is none.
fn to_volume(object: &AlifeObject, graph: &SpawnLevelGraph, level: &str) -> Option<AnnotationVolume> {
  let (kind, shapes, restrictor_type, destination) = match &object.inherited {
    AlifeObjectInherited::SeSmartTerrain(terrain) => {
      let restrictor = &terrain.base.base;

      (
        LevelAnnotationKind::SmartTerrains,
        &restrictor.shape,
        restrictor.restrictor_type,
        None,
      )
    }
    AlifeObjectInherited::CseAlifeSpaceRestrictor(restrictor) => (
      LevelAnnotationKind::Restrictors,
      &restrictor.shape,
      restrictor.restrictor_type,
      None,
    ),
    AlifeObjectInherited::CseAlifeAnomalousZone(zone) => {
      let restrictor = &zone.base.base.base;

      (
        LevelAnnotationKind::Anomalies,
        &restrictor.shape,
        restrictor.restrictor_type,
        None,
      )
    }
    AlifeObjectInherited::CseAlifeTorridZone(zone) => {
      let restrictor = &zone.base.base;

      (
        LevelAnnotationKind::Anomalies,
        &restrictor.shape,
        restrictor.restrictor_type,
        None,
      )
    }
    AlifeObjectInherited::SeLevelChanger(changer) => (
      LevelAnnotationKind::LevelChangers,
      &changer.base.shape,
      changer.base.restrictor_type,
      Some((
        changer.dest_level_name.clone(),
        get_toward(graph, level, &changer.dest_level_name),
      )),
    ),
    _ => return None,
  };
  // `CSpaceRestrictor::net_Spawn`: the shapes stand in the object's own space, its `XFORM` at its place and angle.
  let to_world: VisualTransform = VisualTransform::of_spawn(&object.position, &object.direction).mirrored();

  Some(AnnotationVolume {
    kind,
    name: object.name.clone(),
    outlines: shapes.iter().map(|shape| to_outline(shape, &to_world)).collect(),
    is_in: RESTRICTOR_TYPES_IN.contains(&restrictor_type),
    destination,
  })
}

/// The way from `level` toward `destination` across the world the graph places every level in, as a unit, or `None`
/// where the graph lacks either or places them together.
fn get_toward(graph: &SpawnLevelGraph, level: &str, destination: &str) -> Option<AnnotationPoint> {
  let (from, to) = (
    graph.get_level(level)?.offset.clone(),
    graph.get_level(destination)?.offset.clone(),
  );
  let (x, z): (f32, f32) = (to.x - from.x, to.z - from.z);
  let length: f32 = x.hypot(z);

  (length > f32::EPSILON).then(|| [x / length, z / length])
}

/// A shape from above, through the object's transform.
fn to_outline(shape: &Shape, to_world: &VisualTransform) -> AnnotationOutline {
  match shape {
    Shape::Sphere((center, radius)) => AnnotationOutline::Circle {
      center: to_point(&to_world.apply_to_point(center)),
      radius: *radius,
    },
    Shape::Box((i, j, k, c)) => {
      // The unit cube about the origin through the box's axes and offset, as `CCF_Shape` and the bounding sphere
      // take it.
      let corners: Vec<AnnotationPoint> = [-0.5f32, 0.5]
        .iter()
        .flat_map(|x| {
          [-0.5f32, 0.5]
            .iter()
            .flat_map(move |y| [-0.5f32, 0.5].iter().map(move |z| (*x, *y, *z)))
        })
        .map(|(x, y, z)| {
          Vector3d::new(
            c.x + i.x * x + j.x * y + k.x * z,
            c.y + i.y * x + j.y * y + k.y * z,
            c.z + i.z * x + j.z * y + k.z * z,
          )
        })
        .map(|local| to_point(&to_world.apply_to_point(&local)))
        .collect();

      AnnotationOutline::Polygon(to_convex_hull(corners))
    }
  }
}

/// The convex hull of `points`, counterclockwise from the leftmost, by Andrew's monotone chain.
fn to_convex_hull(mut points: Vec<AnnotationPoint>) -> Vec<AnnotationPoint> {
  points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
  points.dedup();

  if points.len() < 3 {
    return points;
  }

  let cross = |o: AnnotationPoint, a: AnnotationPoint, b: AnnotationPoint| -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
  };
  let mut hull: Vec<AnnotationPoint> = Vec::with_capacity(points.len() + 1);

  // The lower chain left to right, then the upper right to left on top of it, each turning only counterclockwise.
  for point in points.iter().copied() {
    while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
      hull.pop();
    }

    hull.push(point);
  }

  let lower: usize = hull.len() + 1;

  for point in points.iter().rev().skip(1).copied() {
    while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
      hull.pop();
    }

    hull.push(point);
  }

  // The last point is the first again.
  hull.pop();

  hull
}

/// A patrol from above, its links by the points' places.
fn to_patrol(patrol: &Patrol) -> AnnotationPatrol {
  AnnotationPatrol {
    name: patrol.name.clone(),
    points: patrol.points.iter().map(|point| to_point(&point.position)).collect(),
    links: patrol
      .links
      .iter()
      .flat_map(|link| {
        link
          .links
          .iter()
          .map(move |(to, _)| (link.index as usize, *to as usize))
      })
      .filter(|(from, to)| *from < patrol.points.len() && *to < patrol.points.len())
      .collect(),
  }
}
