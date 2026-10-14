use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_composer::compose_shader;

/// Every WGSL module the renderer has, by the name an `#import` gives it: its path under `shaders/`, without the
/// extension.
const EMBEDDED: &[(&str, &str)] = &[
  ("common/camera", include_str!("../../shaders/common/camera.wgsl")),
  ("common/froxel", include_str!("../../shaders/common/froxel.wgsl")),
  (
    "common/compute_grid",
    include_str!("../../shaders/common/compute_grid.wgsl"),
  ),
  (
    "common/fullscreen",
    include_str!("../../shaders/common/fullscreen.wgsl"),
  ),
  (
    "common/light_clusters",
    include_str!("../../shaders/common/light_clusters.wgsl"),
  ),
  (
    "common/local_lights",
    include_str!("../../shaders/common/local_lights.wgsl"),
  ),
  (
    "common/contact_march",
    include_str!("../../shaders/common/contact_march.wgsl"),
  ),
  (
    "common/foliage_wind",
    include_str!("../../shaders/common/foliage_wind.wgsl"),
  ),
  ("common/cut_out", include_str!("../../shaders/common/cut_out.wgsl")),
  ("common/debanding", include_str!("../../shaders/common/debanding.wgsl")),
  ("common/hmodel", include_str!("../../shaders/common/hmodel.wgsl")),
  ("common/lighting", include_str!("../../shaders/common/lighting.wgsl")),
  (
    "common/scene_light",
    include_str!("../../shaders/common/scene_light.wgsl"),
  ),
  ("common/occlusion", include_str!("../../shaders/common/occlusion.wgsl")),
  (
    "common/pyramid_occlusion",
    include_str!("../../shaders/common/pyramid_occlusion.wgsl"),
  ),
  ("common/present", include_str!("../../shaders/common/present.wgsl")),
  (
    "common/reflection_trace",
    include_str!("../../shaders/common/reflection_trace.wgsl"),
  ),
  (
    "common/reflection_end",
    include_str!("../../shaders/common/reflection_end.wgsl"),
  ),
  (
    "common/octahedral",
    include_str!("../../shaders/common/octahedral.wgsl"),
  ),
  (
    "common/puddle_site",
    include_str!("../../shaders/common/puddle_site.wgsl"),
  ),
  ("common/relief", include_str!("../../shaders/common/relief.wgsl")),
  (
    "common/rain_cover",
    include_str!("../../shaders/common/rain_cover.wgsl"),
  ),
  ("common/flares", include_str!("../../shaders/common/flares.wgsl")),
  ("common/sky", include_str!("../../shaders/common/sky.wgsl")),
  (
    "common/scene_trace",
    include_str!("../../shaders/common/scene_trace.wgsl"),
  ),
  (
    "common/scene_trace_none",
    include_str!("../../shaders/common/scene_trace_none.wgsl"),
  ),
  (
    "common/sun_shadow",
    include_str!("../../shaders/common/sun_shadow.wgsl"),
  ),
  ("common/sky_box", include_str!("../../shaders/common/sky_box.wgsl")),
  (
    "common/sky_clouds",
    include_str!("../../shaders/common/sky_clouds.wgsl"),
  ),
  ("common/wet", include_str!("../../shaders/common/wet.wgsl")),
  (
    "common/visibility_bitmask",
    include_str!("../../shaders/common/visibility_bitmask.wgsl"),
  ),
  (
    "common/water_enhanced",
    include_str!("../../shaders/common/water_enhanced.wgsl"),
  ),
  (
    "common/water_surface",
    include_str!("../../shaders/common/water_surface.wgsl"),
  ),
  (
    "frame/ambient_occlusion",
    include_str!("../../shaders/frame/ambient_occlusion.wgsl"),
  ),
  ("frame/combine", include_str!("../../shaders/frame/combine.wgsl")),
  (
    "frame/contact_shadows",
    include_str!("../../shaders/frame/contact_shadows.wgsl"),
  ),
  ("frame/vbao", include_str!("../../shaders/frame/vbao.wgsl")),
  (
    "frame/depth_clear",
    include_str!("../../shaders/frame/depth_clear.wgsl"),
  ),
  (
    "frame/shadow_tile_restore",
    include_str!("../../shaders/frame/shadow_tile_restore.wgsl"),
  ),
  (
    "frame/overhead_shift",
    include_str!("../../shaders/frame/overhead_shift.wgsl"),
  ),
  ("frame/exposure", include_str!("../../shaders/frame/exposure.wgsl")),
  (
    "frame/light_binning",
    include_str!("../../shaders/frame/light_binning.wgsl"),
  ),
  (
    "frame/fsr/accumulate",
    include_str!("../../shaders/frame/fsr/accumulate.wgsl"),
  ),
  ("frame/fsr/common", include_str!("../../shaders/frame/fsr/common.wgsl")),
  (
    "frame/fsr/depth_clip",
    include_str!("../../shaders/frame/fsr/depth_clip.wgsl"),
  ),
  ("frame/fsr/dilate", include_str!("../../shaders/frame/fsr/dilate.wgsl")),
  ("frame/fsr/lock", include_str!("../../shaders/frame/fsr/lock.wgsl")),
  (
    "frame/fsr/luma_first",
    include_str!("../../shaders/frame/fsr/luma_first.wgsl"),
  ),
  (
    "frame/fsr/luma_shading",
    include_str!("../../shaders/frame/fsr/luma_shading.wgsl"),
  ),
  (
    "frame/fsr/nearest",
    include_str!("../../shaders/frame/fsr/nearest.wgsl"),
  ),
  (
    "frame/fsr/reactive",
    include_str!("../../shaders/frame/fsr/reactive.wgsl"),
  ),
  (
    "frame/fsr/reconstruct",
    include_str!("../../shaders/frame/fsr/reconstruct.wgsl"),
  ),
  ("frame/fxaa", include_str!("../../shaders/frame/fxaa.wgsl")),
  ("frame/lights", include_str!("../../shaders/frame/lights.wgsl")),
  ("frame/sky_behind", include_str!("../../shaders/frame/sky_behind.wgsl")),
  ("frame/ray_guides", include_str!("../../shaders/frame/ray_guides.wgsl")),
  ("frame/rtao", include_str!("../../shaders/frame/rtao.wgsl")),
  ("frame/sky_motion", include_str!("../../shaders/frame/sky_motion.wgsl")),
  ("frame/sun_trace", include_str!("../../shaders/frame/sun_trace.wgsl")),
  (
    "frame/light_contact",
    include_str!("../../shaders/frame/light_contact.wgsl"),
  ),
  (
    "frame/light_selection",
    include_str!("../../shaders/frame/light_selection.wgsl"),
  ),
  (
    "frame/light_tracing",
    include_str!("../../shaders/frame/light_tracing.wgsl"),
  ),
  (
    "frame/light_visibility",
    include_str!("../../shaders/frame/light_visibility.wgsl"),
  ),
  ("frame/overlay", include_str!("../../shaders/frame/overlay.wgsl")),
  ("frame/backdrop", include_str!("../../shaders/frame/backdrop.wgsl")),
  ("frame/bloom", include_str!("../../shaders/frame/bloom.wgsl")),
  (
    "frame/enhanced_bloom",
    include_str!("../../shaders/frame/enhanced_bloom.wgsl"),
  ),
  (
    "frame/fog_scattering",
    include_str!("../../shaders/frame/fog_scattering.wgsl"),
  ),
  ("frame/present", include_str!("../../shaders/frame/present.wgsl")),
  ("frame/pyramid", include_str!("../../shaders/frame/pyramid.wgsl")),
  (
    "frame/puddle_sites",
    include_str!("../../shaders/frame/puddle_sites.wgsl"),
  ),
  (
    "frame/sky_prefilter",
    include_str!("../../shaders/frame/sky_prefilter.wgsl"),
  ),
  (
    "frame/surroundings",
    include_str!("../../shaders/frame/surroundings.wgsl"),
  ),
  (
    "frame/reflections",
    include_str!("../../shaders/frame/reflections.wgsl"),
  ),
  (
    "common/trace_requests",
    include_str!("../../shaders/common/trace_requests.wgsl"),
  ),
  (
    "frame/reflection_scene",
    include_str!("../../shaders/frame/reflection_scene.wgsl"),
  ),
  ("frame/particles", include_str!("../../shaders/frame/particles.wgsl")),
  ("frame/rain", include_str!("../../shaders/frame/rain.wgsl")),
  ("frame/sky_haze", include_str!("../../shaders/frame/sky_haze.wgsl")),
  ("frame/smaa", include_str!("../../shaders/frame/smaa.wgsl")),
  ("frame/sun", include_str!("../../shaders/frame/sun.wgsl")),
  ("frame/sun_shafts", include_str!("../../shaders/frame/sun_shafts.wgsl")),
  (
    "frame/light_volumes",
    include_str!("../../shaders/frame/light_volumes.wgsl"),
  ),
  (
    "frame/froxel_inject",
    include_str!("../../shaders/frame/froxel_inject.wgsl"),
  ),
  (
    "frame/froxel_integrate",
    include_str!("../../shaders/frame/froxel_integrate.wgsl"),
  ),
  ("frame/temporal", include_str!("../../shaders/frame/temporal.wgsl")),
  ("frame/flare", include_str!("../../shaders/frame/flare.wgsl")),
  (
    "frame/flare_visibility",
    include_str!("../../shaders/frame/flare_visibility.wgsl"),
  ),
  ("frame/thunder", include_str!("../../shaders/frame/thunder.wgsl")),
  ("frame/upscale", include_str!("../../shaders/frame/upscale.wgsl")),
  ("frame/water_blur", include_str!("../../shaders/frame/water_blur.wgsl")),
  ("frame/wet_apply", include_str!("../../shaders/frame/wet_apply.wgsl")),
  ("frame/wet_patch", include_str!("../../shaders/frame/wet_patch.wgsl")),
  (
    "generated/common/sky",
    include_str!("../../shaders/generated/common/sky.wgsl"),
  ),
  (
    "generated/common/scene_trace",
    include_str!("../../shaders/generated/common/scene_trace.wgsl"),
  ),
  (
    "generated/frame/sun",
    include_str!("../../shaders/generated/frame/sun.wgsl"),
  ),
  (
    "generated/frame/light_binning",
    include_str!("../../shaders/generated/frame/light_binning.wgsl"),
  ),
  (
    "generated/frame/lights",
    include_str!("../../shaders/generated/frame/lights.wgsl"),
  ),
  (
    "generated/grass/occlusion",
    include_str!("../../shaders/generated/grass/occlusion.wgsl"),
  ),
  (
    "generated/frame/sky_behind",
    include_str!("../../shaders/generated/frame/sky_behind.wgsl"),
  ),
  (
    "generated/frame/ray_guides",
    include_str!("../../shaders/generated/frame/ray_guides.wgsl"),
  ),
  (
    "generated/frame/sky_motion",
    include_str!("../../shaders/generated/frame/sky_motion.wgsl"),
  ),
  (
    "generated/frame/sun_trace",
    include_str!("../../shaders/generated/frame/sun_trace.wgsl"),
  ),
  (
    "generated/frame/light_contact",
    include_str!("../../shaders/generated/frame/light_contact.wgsl"),
  ),
  (
    "generated/frame/light_shadowing",
    include_str!("../../shaders/generated/frame/light_shadowing.wgsl"),
  ),
  (
    "generated/frame/light_tracing",
    include_str!("../../shaders/generated/frame/light_tracing.wgsl"),
  ),
  (
    "generated/frame/light_visibility",
    include_str!("../../shaders/generated/frame/light_visibility.wgsl"),
  ),
  (
    "generated/frame/ambient_occlusion",
    include_str!("../../shaders/generated/frame/ambient_occlusion.wgsl"),
  ),
  (
    "generated/frame/vbao",
    include_str!("../../shaders/generated/frame/vbao.wgsl"),
  ),
  (
    "generated/frame/reflections",
    include_str!("../../shaders/generated/frame/reflections.wgsl"),
  ),
  (
    "generated/grass/colours",
    include_str!("../../shaders/generated/grass/colours.wgsl"),
  ),
  (
    "generated/common/trace_requests",
    include_str!("../../shaders/generated/common/trace_requests.wgsl"),
  ),
  (
    "generated/frame/reflection_scene",
    include_str!("../../shaders/generated/frame/reflection_scene.wgsl"),
  ),
  (
    "generated/frame/combine",
    include_str!("../../shaders/generated/frame/combine.wgsl"),
  ),
  (
    "generated/frame/contact_shadows",
    include_str!("../../shaders/generated/frame/contact_shadows.wgsl"),
  ),
  (
    "generated/static/composited",
    include_str!("../../shaders/generated/static/composited.wgsl"),
  ),
  (
    "generated/static/glass",
    include_str!("../../shaders/generated/static/glass.wgsl"),
  ),
  (
    "generated/frame/sky_haze",
    include_str!("../../shaders/generated/frame/sky_haze.wgsl"),
  ),
  (
    "generated/frame/sun_shafts",
    include_str!("../../shaders/generated/frame/sun_shafts.wgsl"),
  ),
  (
    "generated/frame/light_volumes",
    include_str!("../../shaders/generated/frame/light_volumes.wgsl"),
  ),
  (
    "generated/frame/froxel_inject",
    include_str!("../../shaders/generated/frame/froxel_inject.wgsl"),
  ),
  (
    "generated/frame/froxel_integrate",
    include_str!("../../shaders/generated/frame/froxel_integrate.wgsl"),
  ),
  (
    "generated/frame/exposure",
    include_str!("../../shaders/generated/frame/exposure.wgsl"),
  ),
  (
    "generated/frame/flare",
    include_str!("../../shaders/generated/frame/flare.wgsl"),
  ),
  (
    "generated/frame/flare_visibility",
    include_str!("../../shaders/generated/frame/flare_visibility.wgsl"),
  ),
  (
    "generated/frame/rain",
    include_str!("../../shaders/generated/frame/rain.wgsl"),
  ),
  (
    "generated/frame/wet_patch",
    include_str!("../../shaders/generated/frame/wet_patch.wgsl"),
  ),
  (
    "generated/frame/wet_apply",
    include_str!("../../shaders/generated/frame/wet_apply.wgsl"),
  ),
  (
    "generated/frame/thunder",
    include_str!("../../shaders/generated/frame/thunder.wgsl"),
  ),
  (
    "generated/frame/temporal",
    include_str!("../../shaders/generated/frame/temporal.wgsl"),
  ),
  (
    "generated/frame/pyramid",
    include_str!("../../shaders/generated/frame/pyramid.wgsl"),
  ),
  (
    "generated/frame/shadow_tile_restore",
    include_str!("../../shaders/generated/frame/shadow_tile_restore.wgsl"),
  ),
  (
    "generated/frame/overhead_shift",
    include_str!("../../shaders/generated/frame/overhead_shift.wgsl"),
  ),
  (
    "generated/frame/puddle_sites",
    include_str!("../../shaders/generated/frame/puddle_sites.wgsl"),
  ),
  (
    "generated/frame/sky_prefilter",
    include_str!("../../shaders/generated/frame/sky_prefilter.wgsl"),
  ),
  (
    "generated/frame/surroundings",
    include_str!("../../shaders/generated/frame/surroundings.wgsl"),
  ),
  (
    "generated/frame/bloom",
    include_str!("../../shaders/generated/frame/bloom.wgsl"),
  ),
  (
    "generated/frame/enhanced_bloom",
    include_str!("../../shaders/generated/frame/enhanced_bloom.wgsl"),
  ),
  (
    "generated/frame/fog_scattering",
    include_str!("../../shaders/generated/frame/fog_scattering.wgsl"),
  ),
  (
    "generated/frame/fxaa",
    include_str!("../../shaders/generated/frame/fxaa.wgsl"),
  ),
  (
    "generated/frame/overlay",
    include_str!("../../shaders/generated/frame/overlay.wgsl"),
  ),
  (
    "generated/frame/particles",
    include_str!("../../shaders/generated/frame/particles.wgsl"),
  ),
  (
    "generated/frame/present",
    include_str!("../../shaders/generated/frame/present.wgsl"),
  ),
  (
    "generated/frame/smaa",
    include_str!("../../shaders/generated/frame/smaa.wgsl"),
  ),
  (
    "generated/frame/upscale",
    include_str!("../../shaders/generated/frame/upscale.wgsl"),
  ),
  (
    "generated/frame/water_blur",
    include_str!("../../shaders/generated/frame/water_blur.wgsl"),
  ),
  (
    "generated/static/cull",
    include_str!("../../shaders/generated/static/cull.wgsl"),
  ),
  (
    "generated/static/draw",
    include_str!("../../shaders/generated/static/draw.wgsl"),
  ),
  (
    "generated/static/trace_sway",
    include_str!("../../shaders/generated/static/trace_sway.wgsl"),
  ),
  (
    "generated/static/impostor",
    include_str!("../../shaders/generated/static/impostor.wgsl"),
  ),
  (
    "generated/static/water",
    include_str!("../../shaders/generated/static/water.wgsl"),
  ),
  (
    "generated/static/water_enhanced",
    include_str!("../../shaders/generated/static/water_enhanced.wgsl"),
  ),
  (
    "generated/static/water_reflection",
    include_str!("../../shaders/generated/static/water_reflection.wgsl"),
  ),
  (
    "generated/structs",
    include_str!("../../shaders/generated/structs.wgsl"),
  ),
  (
    "grass/card_shadow",
    include_str!("../../shaders/grass/card_shadow.wgsl"),
  ),
  ("grass/cards", include_str!("../../shaders/grass/cards.wgsl")),
  ("grass/colours", include_str!("../../shaders/grass/colours.wgsl")),
  ("grass/grass", include_str!("../../shaders/grass/grass.wgsl")),
  ("grass/planting", include_str!("../../shaders/grass/planting.wgsl")),
  (
    "grass/planting_bindings",
    include_str!("../../shaders/grass/planting_bindings.wgsl"),
  ),
  ("grass/puddles", include_str!("../../shaders/grass/puddles.wgsl")),
  ("grass/records", include_str!("../../shaders/grass/records.wgsl")),
  ("grass/ring", include_str!("../../shaders/grass/ring.wgsl")),
  ("grass/tuft", include_str!("../../shaders/grass/tuft.wgsl")),
  ("grid/grid", include_str!("../../shaders/grid/grid.wgsl")),
  (
    "static/composited",
    include_str!("../../shaders/static/composited.wgsl"),
  ),
  ("static/glass", include_str!("../../shaders/static/glass.wgsl")),
  ("static/cull", include_str!("../../shaders/static/cull.wgsl")),
  ("static/gbuffer", include_str!("../../shaders/static/gbuffer.wgsl")),
  ("static/impostor", include_str!("../../shaders/static/impostor.wgsl")),
  ("static/pulling", include_str!("../../shaders/static/pulling.wgsl")),
  ("static/records", include_str!("../../shaders/static/records.wgsl")),
  ("static/sway", include_str!("../../shaders/static/sway.wgsl")),
  (
    "static/trace_sway",
    include_str!("../../shaders/static/trace_sway.wgsl"),
  ),
  ("static/vertex", include_str!("../../shaders/static/vertex.wgsl")),
  ("static/water", include_str!("../../shaders/static/water.wgsl")),
  (
    "static/water_enhanced",
    include_str!("../../shaders/static/water_enhanced.wgsl"),
  ),
  (
    "static/water_reflection",
    include_str!("../../shaders/static/water_reflection.wgsl"),
  ),
];

/// Modules that need a device feature, each with the module a device without it composes in its place: the same
/// functions over nothing, so a pass importing one draws on every device and does more where the feature is.
const FALLBACKS: &[(&str, wgpu::Features, &str)] = &[(
  "common/scene_trace",
  wgpu::Features::EXPERIMENTAL_RAY_QUERY,
  "common/scene_trace_none",
)];

/// The renderer's WGSL modules: embedded in a release build, read from the crate's `shaders/` directory in a debug
/// one so an edited file is drawn with on the next reload.
#[derive(Clone, Debug)]
pub struct ShaderLibrary {
  modules: BTreeMap<&'static str, String>,
  /// Bumped by every reload that changed a module, so a pass knows its pipelines are stale.
  generation: u64,
  /// When each module's file last changed, for a debug build's reload.
  stamps: BTreeMap<&'static str, Option<SystemTime>>,
}

impl Default for ShaderLibrary {
  fn default() -> Self {
    let mut library: Self = Self {
      modules: EMBEDDED
        .iter()
        .map(|(name, source)| (*name, source.to_string()))
        .collect(),
      generation: 0,
      stamps: BTreeMap::new(),
    };

    if cfg!(debug_assertions) {
      library.reload();
    }

    library
  }
}

impl ShaderLibrary {
  /// The names of every module.
  pub fn list_modules(&self) -> impl Iterator<Item = &'static str> + '_ {
    self.modules.keys().copied()
  }

  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  /// A module with its imports inlined, every feature taken as there.
  ///
  /// # Errors
  ///
  /// Returns an error for an unknown module or import.
  pub fn compose(&self, entry: &str) -> XrfResult<String> {
    self.compose_for(entry, wgpu::Features::all())
  }

  /// A module with its imports inlined for a device with `features`: an import needing one it lacks composed as its
  /// fallback.
  ///
  /// # Errors
  ///
  /// Returns an error for an unknown module or import.
  pub fn compose_for(&self, entry: &str, features: wgpu::Features) -> XrfResult<String> {
    compose_shader(entry, &|name: &str| {
      let name: &str = FALLBACKS
        .iter()
        .find(|(module, needed, _)| *module == name && !features.contains(*needed))
        .map_or(name, |(_, _, fallback)| fallback);

      self
        .modules
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| XrfError::new_not_found_error(format!("No shader module '{name}'")))
    })
  }

  /// Where a debug build reads the modules' sources from: the crate's own `shaders` folder.
  pub fn get_source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders")
  }

  /// Reads again every module whose file changed since it was last read, in a debug build; answers whether any did.
  pub fn reload(&mut self) -> bool {
    if !cfg!(debug_assertions) {
      return false;
    }

    let root: PathBuf = Self::get_source_root();
    let mut is_changed: bool = false;

    for (name, _) in EMBEDDED {
      let path: PathBuf = root.join(format!("{name}.wgsl"));
      let stamp: Option<SystemTime> = std::fs::metadata(&path).and_then(|it| it.modified()).ok();

      if self.stamps.get(name) == Some(&stamp) {
        continue;
      }

      // A file an editor holds mid-save is read again next time, its stamp not yet taken.
      let Ok(source) = std::fs::read_to_string(&path) else {
        continue;
      };

      self.stamps.insert(name, stamp);

      if self.modules.get(name) != Some(&source) {
        log::info!("Shader module '{name}' changed on disk");
        self.modules.insert(name, source);
        is_changed = true;
      }
    }

    if is_changed {
      self.generation += 1;
    }

    is_changed
  }
}
