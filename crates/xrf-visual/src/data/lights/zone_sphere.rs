use serde::Serialize;
use xrf_ltx::Ltx;
use xrf_math::Vector3d;
use xrf_spawn::Shape;

use crate::data::visual::skeleton::visual_transform::VisualTransform;

/// The sphere a zone's distance from the camera is measured to (`CCustomZone::shedule_Update`, the sphere of its
/// `CCF_Shape`): its centre as an offset from what stands with the zone, and its radius.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneSphere {
  pub offset: Vector3d,
  pub radius: f32,
}

impl ZoneSphere {
  /// A spawned zone's, in engine space, its offset from where the zone stands; nothing about the zone itself for one
  /// without shapes, as an empty form's sphere is, and none for one always fast (`[fast_mode] always_fast` in its custom
  /// data), which never switches.
  pub fn of_zone(shapes: &[Shape], (position, angle): (&Vector3d, &Vector3d), custom_data: &str) -> Option<Self> {
    if Self::is_always_fast(custom_data) {
      return None;
    }

    let (center, radius) = Shape::get_bounding_sphere(shapes).unwrap_or((Vector3d::new(0.0, 0.0, 0.0), 0.0));
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
    let center: [f32; 3] = [at[0] + self.offset.x, at[1] + self.offset.y, at[2] + self.offset.z];
    let squared: f32 = (0..3).map(|axis| (point[axis] - center[axis]).powi(2)).sum();

    squared.sqrt() - self.radius
  }

  /// Whether the custom data writes `always_fast` on in its `[fast_mode]` section, as `CCustomZone::net_Spawn` reads.
  fn is_always_fast(custom_data: &str) -> bool {
    !custom_data.trim().is_empty()
      && Ltx::read_from_str(custom_data)
        .ok()
        .and_then(|ltx| {
          ltx
            .section("fast_mode")
            .and_then(|section| section.get_bool("always_fast"))
        })
        .unwrap_or(false)
  }
}
