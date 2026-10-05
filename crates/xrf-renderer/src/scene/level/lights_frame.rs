use crate::camera::camera_view::CameraView;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::scene::level::level_campfires::LevelCampfires;
use crate::scene::level::level_object_motions::LevelObjectMotions;
use crate::scene::level::shadow_sway::ShadowSway;

/// What a level's frame hands its local lights: the camera and settings, what has their shadow faces drawn again, the
/// campfires fading their idle lights and the motions carrying moving zones' ones.
pub struct LightsFrame<'a> {
  pub camera: &'a CameraView,
  pub settings: &'a RenderLightsSettings,
  /// The progressive meshes' `start` and `end` screen areas, which a shadowed light fades between.
  pub lod: (f32, f32),
  /// What the scene holds, which a face drawn with less is drawn again for.
  pub contents: usize,
  /// How the trees sway, which has a face over them drawn again.
  pub sway: &'a ShadowSway<'a>,
  pub campfires: &'a mut LevelCampfires,
  pub motions: &'a mut LevelObjectMotions,
}
