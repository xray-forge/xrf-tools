use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelOptions;
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelLightsFile};

use crate::level::level_glow_conversion::LevelGlowConversion;
use crate::level::level_light_conversion::LevelLightConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneInference;

/// What the compiled bundle and the level compiler's light list give besides geometry: glows, lights, and the light
/// qualities of the level options, from the generated sky and sun.
pub(crate) struct LevelBundleStep;

impl LevelSceneStep for LevelBundleStep {
  const FILES: &'static [&'static str] = &[LevelFile::FILE_NAME, LevelLightsFile::FILE_NAME];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(bundle) = sources.get_bundle::<T>()? else {
      return Ok(());
    };
    let lights: Option<LevelLightsFile> =
      sources.take_read(LevelLightsFile::FILE_NAME, LevelLightsFile::read_from_bytes::<T>)?;

    if let Some(lights) = &lights {
      let options: &mut EditableLevelOptions = &mut draft.level_options;

      options.light_hemi_quality = LevelLightConversion::get_hemi_quality(lights).unwrap_or(options.light_hemi_quality);
      options.light_sun_quality = LevelLightConversion::get_sun_quality(lights).unwrap_or(options.light_sun_quality);
      draft.add_level_source(LevelLightsFile::FILE_NAME, sources);
    }

    let glows = LevelGlowConversion::decompile(&bundle.file, &mut draft.context)?;

    draft.infer_if(
      !glows.objects.is_empty(),
      &[LevelSceneInference::GlowTransformsAndFlags],
    );
    draft.add(glows.to_scene_tool()?, &[LevelFile::FILE_NAME], sources);

    let lights = LevelLightConversion::decompile(lights.as_ref(), &bundle.file, &mut draft.context);

    draft.infer_if(
      !lights.objects.is_empty(),
      &[
        LevelSceneInference::LightBrightness,
        LevelSceneInference::LightFuzzyPoints,
        LevelSceneInference::LightControls,
      ],
    );
    draft.add(lights.to_scene_tool()?, Self::FILES, sources);

    Ok(())
  }
}
