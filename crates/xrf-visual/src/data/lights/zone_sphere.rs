use xrf_math::Vector3d;
use xrf_spawn::{AlifeObjectCustomZone, Shape};

use crate::data::visual::skeleton::visual_transform::VisualTransform;

/// The sphere a zone's distance from the camera is measured to (`CCustomZone::shedule_Update`, the sphere of its
/// `CCF_Shape`): its centre as an offset from what stands with the zone, and its radius.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneSphere {
  pub offset: Vector3d,
  pub radius: f32,
}

impl ZoneSphere {
  /// A spawned zone's standing at `position`, turned by `angle`, in engine space, its offset from where the zone stands;
  /// nothing about the zone itself for one without shapes, as an empty form's sphere is. None for a zone always fast,
  /// which never switches and so keeps no sphere to measure.
  pub fn of_custom_zone(zone: &AlifeObjectCustomZone, (position, angle): (&Vector3d, &Vector3d)) -> Option<Self> {
    if zone.is_always_fast() {
      return None;
    }

    let (center, radius) = Shape::get_bounding_sphere(&zone.base.shape).unwrap_or((Vector3d::new(0.0, 0.0, 0.0), 0.0));
    // `XFORM().transform_tiny(P, s.P)`, in engine space.
    let at: Vector3d = VisualTransform::of_spawn(position, angle)
      .mirrored()
      .apply_to_point(&center);

    Some(Self {
      offset: Vector3d::new(at.x - position.x, at.y - position.y, at.z - position.z),
      radius,
    })
  }

  /// The same sphere offset from a point standing `lift` above the zone, mirrored into renderer space.
  pub fn to_renderer_space(&self, lift: f32) -> Self {
    Self {
      offset: Vector3d::new(self.offset.x, self.offset.y - lift, -self.offset.z),
      radius: self.radius,
    }
  }

  /// How far a point stands outside it, `distance_to(P) - s.R`, where the point it is offset from stands at `at`.
  pub fn get_distance(&self, at: [f32; 3], point: [f32; 3]) -> f32 {
    let center: Vector3d = &Vector3d::from(at) + &self.offset;

    Vector3d::from(point).distance(&center) - self.radius
  }
}
