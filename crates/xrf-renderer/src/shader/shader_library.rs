use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_composer::compose_shader;

/// Every WGSL module the renderer has, by the name an `#import` gives it: its path under `shaders/`, without the
/// extension.
const EMBEDDED: &[(&str, &str)] = &[
  ("common/camera", include_str!("../../shaders/common/camera.wgsl")),
  (
    "common/fullscreen",
    include_str!("../../shaders/common/fullscreen.wgsl"),
  ),
  (
    "common/light_clusters",
    include_str!("../../shaders/common/light_clusters.wgsl"),
  ),
  ("common/cut_out", include_str!("../../shaders/common/cut_out.wgsl")),
  ("common/hmodel", include_str!("../../shaders/common/hmodel.wgsl")),
  ("common/lighting", include_str!("../../shaders/common/lighting.wgsl")),
  ("common/occlusion", include_str!("../../shaders/common/occlusion.wgsl")),
  ("common/present", include_str!("../../shaders/common/present.wgsl")),
  (
    "common/octahedral",
    include_str!("../../shaders/common/octahedral.wgsl"),
  ),
  (
    "common/rain_cover",
    include_str!("../../shaders/common/rain_cover.wgsl"),
  ),
  ("common/sky", include_str!("../../shaders/common/sky.wgsl")),
  (
    "common/sun_shadow",
    include_str!("../../shaders/common/sun_shadow.wgsl"),
  ),
  ("common/sky_box", include_str!("../../shaders/common/sky_box.wgsl")),
  (
    "frame/ambient_occlusion",
    include_str!("../../shaders/frame/ambient_occlusion.wgsl"),
  ),
  ("frame/combine", include_str!("../../shaders/frame/combine.wgsl")),
  (
    "frame/depth_clear",
    include_str!("../../shaders/frame/depth_clear.wgsl"),
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
  ("frame/overlay", include_str!("../../shaders/frame/overlay.wgsl")),
  ("frame/present", include_str!("../../shaders/frame/present.wgsl")),
  ("frame/pyramid", include_str!("../../shaders/frame/pyramid.wgsl")),
  ("frame/rain", include_str!("../../shaders/frame/rain.wgsl")),
  ("frame/sky_haze", include_str!("../../shaders/frame/sky_haze.wgsl")),
  ("frame/smaa", include_str!("../../shaders/frame/smaa.wgsl")),
  ("frame/sun", include_str!("../../shaders/frame/sun.wgsl")),
  ("frame/temporal", include_str!("../../shaders/frame/temporal.wgsl")),
  ("frame/thunder", include_str!("../../shaders/frame/thunder.wgsl")),
  ("frame/upscale", include_str!("../../shaders/frame/upscale.wgsl")),
  ("frame/wet_apply", include_str!("../../shaders/frame/wet_apply.wgsl")),
  ("frame/wet_patch", include_str!("../../shaders/frame/wet_patch.wgsl")),
  ("grass/grass", include_str!("../../shaders/grass/grass.wgsl")),
  ("grass/planting", include_str!("../../shaders/grass/planting.wgsl")),
  ("grass/records", include_str!("../../shaders/grass/records.wgsl")),
  ("grid/grid", include_str!("../../shaders/grid/grid.wgsl")),
  (
    "static/composited",
    include_str!("../../shaders/static/composited.wgsl"),
  ),
  ("static/cull", include_str!("../../shaders/static/cull.wgsl")),
  ("static/gbuffer", include_str!("../../shaders/static/gbuffer.wgsl")),
  ("static/impostor", include_str!("../../shaders/static/impostor.wgsl")),
  ("static/pulling", include_str!("../../shaders/static/pulling.wgsl")),
  ("static/records", include_str!("../../shaders/static/records.wgsl")),
  ("static/water", include_str!("../../shaders/static/water.wgsl")),
];

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

  /// A module with its imports inlined and its `#if` blocks resolved against `defines`.
  ///
  /// # Errors
  ///
  /// Returns an error for an unknown module or import, or an unbalanced `#if`.
  pub fn compose(&self, entry: &str, defines: &[&str]) -> XrfResult<String> {
    compose_shader(entry, defines, &|name: &str| {
      self
        .modules
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| XrfError::new_not_found_error(format!("No shader module '{name}'")))
    })
  }

  /// Reads again every module whose file changed since it was last read, in a debug build; answers whether any did.
  pub fn reload(&mut self) -> bool {
    if !cfg!(debug_assertions) {
      return false;
    }

    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let mut is_changed: bool = false;

    for (name, _) in EMBEDDED {
      let path: PathBuf = root.join(format!("{name}.wgsl"));
      let stamp: Option<SystemTime> = std::fs::metadata(&path).and_then(|it| it.modified()).ok();

      if self.stamps.get(name) == Some(&stamp) {
        continue;
      }

      self.stamps.insert(name, stamp);

      if let Ok(source) = std::fs::read_to_string(&path)
        && self.modules.get(name) != Some(&source)
      {
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
