use glam::Vec3;

/// Distance fog, in the terms a weather keyframe uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFog {
  /// `fog_color`.
  pub color: Vec3,
  /// `fog_density`: how near the camera it starts, a share of its distance.
  pub density: f32,
  /// `fog_distance`: where it is total.
  pub distance: f32,
  /// `far_plane`: where the engine stops drawing.
  pub far_plane: f32,
}

impl RenderFog {
  /// How far it is total, or the far plane where that is nearer: nothing past it shows but the sky.
  pub fn get_total_distance(&self) -> f32 {
    (0.99 * self.distance).min(self.far_plane).max(1.0)
  }

  /// `fog_params.x` and `.w`: fog is `saturate(distance * w + x)`, from `CEnvDescriptorMixer::lerp`'s near and far.
  pub fn get_params(&self) -> (f32, f32) {
    let near: f32 = (1.0 - self.density) * 0.85 * self.distance;
    let far: f32 = 0.99 * self.distance;
    let range: f32 = 1.0 / (far - near).max(f32::EPSILON);

    (-near * range, range)
  }
}
