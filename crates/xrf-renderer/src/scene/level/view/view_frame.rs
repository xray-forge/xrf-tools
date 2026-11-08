use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::RenderShadowSettings;
use crate::feature::image::exposure::reading::exposure_decisions::ExposureDecisions;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// The values every unit of a view reads this frame, prepared before them: what its exposure and resolve decided and how
/// the present shows the frame by them, the camera and the sun it is drawn by, the shading switches, the shadows asked
/// for, whether it draws its edges alone, and the lighting uniform.
pub(crate) struct ViewFrame {
  /// What the exposure decided, prepared before every other unit.
  pub exposure: ExposureDecisions,
  /// What the resolve decided, prepared before every other unit but the exposure.
  pub resolve: ResolveDecisions,
  /// How the present shows the frame (`DisplayMode::get_display`) by what the exposure metered, which the lighting and
  /// the falling rain enter the game's pictures through.
  pub display: Vec4,
  pub camera: CameraView,
  /// Where the sunlight travels.
  pub sun_direction: Vec3,
  /// What the static scene is drawn with off the viewport (the surroundings cube): textured, the viewport's bumping
  /// and the baked hemisphere's strength.
  pub switches: Vec4,
  pub shadow_settings: RenderShadowSettings,
  /// Whether the static surfaces draw as their edges, which nothing composited or planted is drawn over.
  pub is_wireframe: bool,
  /// The lighting the passes read, as the frame's weather and options make it.
  pub lighting: LightingUniform,
}

impl Default for ViewFrame {
  fn default() -> Self {
    Self {
      exposure: ExposureDecisions::default(),
      resolve: ResolveDecisions::default(),
      display: Vec4::ZERO,
      camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      sun_direction: RenderLighting::default().get_sun_direction(),
      switches: Vec4::new(1.0, 1.0, 1.0, 0.0),
      shadow_settings: RenderShadowSettings::default(),
      is_wireframe: false,
      lighting: bytemuck::Zeroable::zeroed(),
    }
  }
}
