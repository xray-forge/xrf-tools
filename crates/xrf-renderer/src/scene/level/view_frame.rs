use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::lighting_uniform::LightingUniform;

/// The values every unit of a view reads this frame, prepared before them: the camera and the sun it is drawn by, the
/// shading switches, the shadows asked for, whether it draws solid and shafted, and the lighting uniform.
pub struct ViewFrame {
  pub camera: CameraView,
  /// Where the sunlight travels.
  pub sun_direction: Vec3,
  /// What the static scene is drawn with off the viewport (the surroundings cube): textured, the viewport's bumping
  /// and the baked hemisphere's strength.
  pub switches: Vec4,
  pub shadow_settings: RenderShadowSettings,
  /// Whether the static surfaces draw as their edges, which nothing composited or planted is drawn over.
  pub is_wireframe: bool,
  /// Whether the sun's light shafts are added, drawn through its shadow's cascades.
  pub is_shafted: bool,
  /// The lighting the passes read, as the frame's weather and options make it.
  pub lighting: LightingUniform,
}

impl Default for ViewFrame {
  fn default() -> Self {
    Self {
      camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      sun_direction: RenderLighting::default().get_sun_direction(),
      switches: Vec4::new(1.0, 1.0, 1.0, 0.0),
      shadow_settings: RenderShadowSettings::default(),
      is_wireframe: false,
      is_shafted: false,
      lighting: bytemuck::Zeroable::zeroed(),
    }
  }
}
