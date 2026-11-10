use byteorder::ByteOrder;
use xrf_editable_level::{EditableObjectPart, EditableShapeObject};
use xrf_error::XrfResult;

use crate::level::level_map_conversion::{LevelMapConversion, LevelMapSettings};
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::{LevelSceneInference, LevelSceneRefusal};

/// What `level.ltx` gives ([`LevelMapConversion`]): the map's version and game types into the level options, the map's
/// rectangle as a level-bound shape, and each sector's part of the map, which the draft keeps for the geometry step's
/// sectors. It runs before the geometry step, which makes the sectors.
pub(crate) struct LevelMapStep;

impl LevelSceneStep for LevelMapStep {
  const FILES: &'static [&'static str] = &[LevelMapConversion::FILE_NAME];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(settings) = sources.take_read(LevelMapConversion::FILE_NAME, |bytes| {
      LevelMapConversion::decompile(&bytes)
    })?
    else {
      return Ok(());
    };

    if let Some(version) = &settings.map_version {
      draft.level_options.map_version.clone_from(version);
    }

    if let Some(bounds) = &settings.bounds {
      let part: EditableObjectPart<EditableShapeObject> = EditableObjectPart::new(
        draft.context.guid,
        vec![LevelMapConversion::decompile_bound(bounds, &mut draft.context)],
      );

      draft.infer_if(true, &[LevelSceneInference::MapBound]);
      draft.add(part.to_scene_tool()?, Self::FILES, sources);
    }

    Self::refuse(draft, &settings);
    draft.add_level_source(LevelMapConversion::FILE_NAME, sources);
    draft.map = Some(settings);

    Ok(())
  }
}

impl LevelMapStep {
  /// Records what `settings` holds that the scene cannot: sections the Build did not write itself, which its build
  /// options' text would carry but a part cannot hold as several lines; game types it does not write; and a texture
  /// other than the one it names after the level.
  fn refuse(draft: &mut LevelSceneDraft, settings: &LevelMapSettings) {
    let refusal = |object: &str, reason: &str| LevelSceneRefusal {
      file: String::from(LevelMapConversion::FILE_NAME),
      object: object.to_owned(),
      reason: reason.to_owned(),
    };

    for section in &settings.other_sections {
      draft.refused.push(refusal(
        &format!("[{section}]"),
        "A section the Build writes only from the build options' text, which the scene's main file cannot hold as \
         several lines",
      ));
    }

    for name in &settings.unknown_game_types {
      draft
        .refused
        .push(refusal(&format!("[map_usage] {name}"), "No game type the Build writes"));
    }

    if let Some(texture) = &settings.texture
      && *texture != LevelMapConversion::get_texture(&draft.level_options.level_path)
    {
      draft.refused.push(refusal(
        &format!("[level_map] texture = {texture}"),
        "The Build names the map's texture after the level's directory",
      ));
    }
  }
}
