use glam::Vec3;
use xrf_math::{Aabb, Affine3d};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, AlifeObjectSpaceRestrictor, Shape};

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_restrictor_shape::SpawnRestrictorShape;

/// A space restrictor xrAI checks for cutting the AI map apart (`add_space_restrictor`, `level_spawn_constructor.cpp`):
/// its shapes placed by the object's position and angles, and which nodes they hold, as `CSpaceRestrictorWrapper`
/// tests them.
pub(crate) struct SpawnRestrictor {
  pub(crate) name: String,
  shapes: Vec<SpawnRestrictorShape>,
}

impl SpawnRestrictor {
  /// `flCheckForSeparator`.
  const CHECK_FOR_SEPARATOR: u32 = 1 << 12;
  /// `eRestrictorTypeNone`, a restrictor restricting nothing.
  const TYPE_NONE: u8 = 3;
  /// The points of a node the shapes are tested at: its centre and its four corners.
  pub(crate) const NODE_POINTS: u8 = 5;

  /// The restrictor of `object` where xrAI checks it: a restrictor of some type, flagged for the check.
  pub(crate) fn from_object(object: &AlifeObject) -> Option<Self> {
    let restrictor: &AlifeObjectSpaceRestrictor = Self::get_restrictor(object)?;

    if restrictor.restrictor_type == Self::TYPE_NONE || restrictor.base.flags & Self::CHECK_FOR_SEPARATOR == 0 {
      return None;
    }

    let transform: Affine3d = Affine3d::from_xyz(&object.header.direction).with_translation(object.header.position);

    Some(Self {
      name: object.header.name.clone(),
      shapes: restrictor
        .shape
        .iter()
        .map(|shape| match shape {
          Shape::Sphere(sphere) => SpawnRestrictorShape::new_sphere(&transform, &sphere.center, sphere.radius),
          Shape::Box(matrix) => SpawnRestrictorShape::new_box(&transform, matrix),
        })
        .collect(),
    })
  }

  /// The box every shape lies within; `None` for a restrictor of no shape.
  pub(crate) fn get_bounds(&self) -> Option<Aabb> {
    let mut shapes = self.shapes.iter().map(SpawnRestrictorShape::get_bounds);
    let mut bounds: Aabb = shapes.next()?;

    shapes.for_each(|shape| bounds.include_box(&shape));

    Some(bounds)
  }

  /// `inside(level_vertex_id, partially_inside)`: how many of the node's points a shape holds, its centre and its four
  /// corners pulled `EPS_L` in and raised to its plane; any for a node held in part, [`Self::NODE_POINTS`] for one held
  /// whole.
  pub(crate) fn count_held_points(&self, map: &SpawnAiMap, node: u32) -> u8 {
    let centre: Vec3 = Vec3::from(map.get_node_position(node));
    let offset: f32 = map.get_header().size * 0.5 - SpawnRestrictorShape::EPS_L;
    let corner = |dx: f32, dz: f32| {
      let (x, z) = (centre.x + dx, centre.z + dz);

      Vec3::new(x, map.get_plane_y(node, x, z), z)
    };
    let points: [Vec3; 5] = [
      corner(offset, offset),
      corner(offset, -offset),
      corner(-offset, offset),
      corner(-offset, -offset),
      centre,
    ];

    points
      .iter()
      .filter(|point| self.shapes.iter().any(|shape| shape.contains(**point)))
      .count() as u8
  }

  /// The restrictor part of every class built on `CSE_ALifeSpaceRestrictor`.
  fn get_restrictor(object: &AlifeObject) -> Option<&AlifeObjectSpaceRestrictor> {
    match &object.inherited {
      AlifeObjectInherited::CseAlifeSpaceRestrictor(object) => Some(object),
      AlifeObjectInherited::SeLevelChanger(object) => Some(&object.base),
      AlifeObjectInherited::CseAlifeTeamBaseZone(object) => Some(&object.base),
      AlifeObjectInherited::SeSmartTerrain(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeTorridZone(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeAnomalousZone(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeZoneVisual(object) => Some(&object.base.base.base),
      AlifeObjectInherited::ClearSkySeSmartTerrain(object) => Some(&object.base.base),
      AlifeObjectInherited::ClearSkySeSimFaction(object) => Some(&object.base.base),
      _ => None,
    }
  }
}
