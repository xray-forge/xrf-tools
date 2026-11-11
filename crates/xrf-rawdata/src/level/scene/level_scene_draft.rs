use xrf_editable_level::{
  EditableBuildParams, EditableLevelCamera, EditableLevelFile, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableSceneTool,
};
use xrf_utils::format_path;

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::level::level_map_conversion::LevelMapSettings;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_decompile::{LevelSceneDecompile, LevelSceneObjectFile, LevelSceneTextureFile};
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_root_spawn::LevelRootSpawn;
use crate::provenance::level_scene_provenance::{
  LevelSceneFile, LevelSceneGeometry, LevelSceneInference, LevelSceneProvenance, LevelSceneRefusal,
};
use crate::provenance::object_provenance::ObjectProvenance;

/// A scene being decompiled, as the steps build it: the context its parts share, its level options and the files they
/// came from, each tool with the files it came from, the objects it places, and what was inferred and refused.
pub(crate) struct LevelSceneDraft {
  pub(crate) context: LevelDecompileContext,
  pub(crate) level_options: EditableLevelOptions,
  /// The files the level options were read from, which the main file comes from.
  level_sources: Vec<&'static str>,
  tools: Vec<(EditableSceneTool, Vec<&'static str>)>,
  inferred: Vec<LevelSceneInference>,
  pub(crate) refused: Vec<LevelSceneRefusal>,
  pub(crate) objects: Vec<LevelDecompiledObject>,
  /// The textures the objects need for the Build.
  pub(crate) textures: Vec<LevelSceneTextureFile>,
  pub(crate) geometry: Option<LevelSceneGeometry>,
  /// The names of the entities the objects stand for, which the Build exports again from them.
  pub(crate) folded_entities: Vec<String>,
  /// The root spawn the entities come from, where one was given.
  pub(crate) root_spawn: Option<LevelRootSpawn>,
  /// What `level.ltx` says, where the directory holds one: the sectors' parts of the map and the game types.
  pub(crate) map: Option<LevelMapSettings>,
}

impl LevelSceneDraft {
  /// The owner the scene's `[level_tag]` names.
  pub(crate) const OWNER: &'static str = "xrf";

  /// A scene of `guid` named `name`, holding no tool yet, with the inferences every decompiled scene makes; the game
  /// types are inferred only where no `level.ltx` names them.
  pub(crate) fn new(guid: EditableLevelGuid, name: &str, root_spawn: Option<LevelRootSpawn>) -> Self {
    Self {
      context: LevelDecompileContext::new(guid),
      level_options: EditableLevelOptions::new(name, EditableLevelOptions::SINGLE_PLAYER),
      level_sources: Vec::new(),
      tools: Vec::new(),
      inferred: vec![
        LevelSceneInference::ObjectNames,
        LevelSceneInference::LevelOptions,
        LevelSceneInference::BuildParameters,
        LevelSceneInference::Camera,
      ],
      refused: Vec::new(),
      objects: Vec::new(),
      textures: Vec::new(),
      geometry: None,
      folded_entities: Vec::new(),
      root_spawn,
      map: None,
    }
  }

  /// Records `inferences` where a part holds what they were made for.
  pub(crate) fn infer_if(&mut self, condition: bool, inferences: &[LevelSceneInference]) {
    if condition {
      self.inferred.extend_from_slice(inferences);
    }
  }

  /// Adds a tool decompiled from those of `files` the directory held.
  pub(crate) fn add(&mut self, tool: EditableSceneTool, files: &[&'static str], sources: &LevelSceneSources) {
    self.tools.push((tool, sources.get_held(files)));
  }

  /// Records that the level options were read from `file`, where the directory held it.
  pub(crate) fn add_level_source(&mut self, file: &'static str, sources: &LevelSceneSources) {
    if sources.holds(file) && !self.level_sources.contains(&file) {
      self.level_sources.push(file);
    }
  }

  /// The scene and its provenance: its tools sorted by name, the main file of the level options the steps gave, and
  /// each object with a sidecar naming the compiled file it came from.
  pub(crate) fn finish(mut self, options: &LevelDecompileOptions, sources: &LevelSceneSources) -> LevelSceneDecompile {
    let objects: Vec<LevelSceneObjectFile> = std::mem::take(&mut self.objects)
      .into_iter()
      .map(|object| LevelSceneObjectFile {
        provenance: ObjectProvenance::of_source_hash(
          format_path(&sources.get_path(&object.source)).to_string(),
          sources.get_sha256(&object.source).unwrap_or_default().to_owned(),
          options.tool.clone(),
          options.tool_version.clone(),
          object.decompile.inferred,
          object.decompile.tangents,
        ),
        reference: object.reference,
        object: object.decompile.object,
      })
      .collect();

    self
      .tools
      .sort_by(|(first, _), (second, _)| first.name.cmp(&second.name));

    let files: Vec<LevelSceneFile> = std::iter::once(LevelSceneFile {
      written: format!("{}.{}", options.name, EditableLevelFile::EXTENSION),
      sources: self.level_sources.iter().map(|name| (*name).to_owned()).collect(),
    })
    .chain(self.tools.iter().map(|(tool, files)| LevelSceneFile {
      written: format!("{}/{}", options.name, tool.get_file_name(0)),
      sources: files.iter().map(|name| (*name).to_owned()).collect(),
    }))
    .collect();
    let level: EditableLevelFile = EditableLevelFile::new(
      &self.context.guid,
      &self.level_options,
      &EditableBuildParams {
        weld_distance: 0.0,
        ..EditableBuildParams::DEFAULT
      },
      &EditableLevelCamera::default(),
      Self::OWNER,
      0,
    );

    LevelSceneDecompile {
      scene: EditableLevelScene {
        level,
        tools: self.tools.into_iter().map(|(tool, _)| tool).collect(),
      },
      provenance: LevelSceneProvenance {
        sources: sources.get_recorded().to_vec(),
        files,
        inferred: self.inferred,
        refused: self.refused,
        root_spawn: self.root_spawn.map(|root| root.record),
        geometry: self.geometry,
        ..LevelSceneProvenance::new(options.name.clone(), options.tool.clone(), options.tool_version.clone())
      },
      objects,
      textures: self.textures,
    }
  }
}
