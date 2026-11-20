use xrf_error::XrfResult;

use crate::feature::effects::effect_passes::EffectPasses;
use crate::feature::image::image_passes::ImagePasses;
use crate::feature::image::present::present_parameters::PresentParameters;
use crate::feature::image::present::present_uniform::PresentUniform;
use crate::feature::lighting::lighting_passes::LightingPasses;
use crate::feature::scene::scene_passes::ScenePasses;
use crate::feature::tracing::tracing_passes::TracingPasses;
use crate::feature::weather::weather_passes::WeatherPasses;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::pass::overhead::overhead_shift_parameters::OverheadShiftParameters;
use crate::pass::overhead::overhead_shift_uniform::OverheadShiftUniform;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::window::backdrop_uniform::BackdropUniform;
use crate::pass::window::overlay_parameters::OverlayParameters;
use crate::scene::level::lights::light_record::LightRecord;
use crate::scene::level::lights::light_volume_record::LightVolumeRecord;
use crate::scene::level::lights::light_volumes_uniform::LightVolumesUniform;
use crate::scene::level::particles::particle_surface_record::ParticleSurfaceRecord;
use crate::scene::level::particles::particle_vertex::ParticleVertex;
use crate::shader::generated_modules::GeneratedModules;

/// The modules under `shaders/generated/`, written from Rust declarations, by name: the structs shaders share with
/// Rust, and each module's bindings, gathered from the parameters of every pass drawing with it, importing the structs.
/// Each group of passes declares its own; the window's passes and what no one group owns are declared here.
///
/// # Errors
///
/// Returns an error when passes sharing a module declare one of its bindings otherwise.
pub(crate) fn list_generated_shaders() -> XrfResult<Vec<(&'static str, String)>> {
  let mut modules: GeneratedModules = GeneratedModules::new();

  modules
    .declare::<CameraUniform>()
    .declare::<BackdropUniform>()
    .declare::<LightingUniform>()
    .declare::<PresentUniform>()
    .declare::<OverheadShiftUniform>()
    .declare::<ParticleVertex>()
    .declare::<ParticleSurfaceRecord>()
    .declare::<LightRecord>()
    .declare::<LightVolumeRecord>()
    .declare::<LightVolumesUniform>();
  modules
    .add::<PresentParameters<'_>>("generated/frame/present")?
    .add::<OverlayParameters>("generated/frame/overlay")?
    .add::<OverheadShiftParameters>("generated/frame/overhead_shift")?;

  ScenePasses::declare_shaders(&mut modules)?;
  LightingPasses::declare_shaders(&mut modules)?;
  TracingPasses::declare_shaders(&mut modules)?;
  WeatherPasses::declare_shaders(&mut modules)?;
  EffectPasses::declare_shaders(&mut modules)?;
  ImagePasses::declare_shaders(&mut modules)?;

  Ok(modules.to_modules())
}
