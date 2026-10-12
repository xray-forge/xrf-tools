use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_debanding_quality::RenderDebandingQuality;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::wind_uniform::WindUniform;

/// One frame of a view as its preparation leaves it: the camera and sun it is drawn by, what its options and weather
/// decided it draws, and what its passes read of that. Prepared before the frame graph is declared, and read by it.
pub struct ViewInfo {
  /// What the present pass shows this frame.
  pub present: PresentUniform,
  pub camera: CameraView,
  /// The camera's view and projection, which become the depth history once the pyramid is reduced.
  pub matrices: (Mat4, Mat4),
  /// Where the sunlight travels.
  pub sun_direction: Vec3,
  /// What the static scene is drawn with off the viewport (the surroundings cube): textured, the viewport's bumping
  /// and the baked hemisphere's strength.
  pub switches: Vec4,
  /// The wind's amplitude and the time the trees sway by: what has still shadows drawn again.
  pub sway: (f32, f32),
  /// The cull's parameters for this frame's view and options.
  pub cull: StaticCullParams,
  /// The view the depth pyramid was reduced through, which the early cull tests against.
  pub occlusion: StaticOcclusionUniform,
  /// How the trees sway this frame, which every static draw reads.
  pub wind: WindUniform,
  /// The viewport's rectangle in its window, which the frame is upscaled to where it is drawn smaller.
  pub output: RenderRect,
  pub shadow_settings: RenderShadowSettings,
  pub lights_settings: RenderLightsSettings,
  /// What corrects the finished image, and what the present shows.
  pub corrections: RenderImageCorrections,
  pub debug_view: RenderDebugView,
  /// Whether the static surfaces draw as their edges, which nothing composited or planted is drawn over.
  pub is_wireframe: bool,
  /// Whether the wall marks are laid into the albedo.
  pub is_wallmarked: bool,
  /// Whether the sun's light shafts are added, drawn through its shadow's cascades.
  pub is_shafted: bool,
  /// How hard the sky is debanded, none where it is drawn as it is.
  pub debanding: Option<RenderDebandingQuality>,
  /// The lighting the passes read, as the frame's weather and options make it.
  pub lighting: LightingUniform,
  /// The colour the selection is outlined in, or none while nothing it names is drawn.
  pub selection_color: Option<[f32; 3]>,
}

impl Default for ViewInfo {
  fn default() -> Self {
    Self {
      present: PresentUniform::default(),
      camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      matrices: (Mat4::IDENTITY, Mat4::IDENTITY),
      sun_direction: RenderLighting::default().get_sun_direction(),
      switches: Vec4::new(1.0, 1.0, 1.0, 0.0),
      sway: (0.0, 0.0),
      cull: StaticCullParams::default(),
      occlusion: bytemuck::Zeroable::zeroed(),
      wind: WindUniform::default(),
      output: RenderRect::default(),
      shadow_settings: RenderShadowSettings::default(),
      lights_settings: RenderLightsSettings::default(),
      corrections: RenderImageCorrections::default(),
      debug_view: RenderDebugView::Final,
      is_wireframe: false,
      is_wallmarked: true,
      is_shafted: false,
      debanding: None,
      lighting: bytemuck::Zeroable::zeroed(),
      selection_color: None,
    }
  }
}
