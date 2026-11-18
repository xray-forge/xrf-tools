use std::collections::BTreeMap;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableLevelFile, EditableLevelGuid, EditableLevelPart, EditableLevelScene, EditableScenePartFile,
  EditableScenePartSource, EditableSceneRead, EditableSceneTool,
};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAsset, XrayInputRecord, XrayRoots, XraySearch};

use crate::scene::{
  SceneGameData, ScenePartFileDescription, SceneProblem, SceneProblemKind, SceneSource, SceneToolDescription,
  SceneToolFormat, SceneToolKind, SelectedScene,
};

/// A Level Editor scene to read through the shared asset state, on open or after a commit moved one of its files: its
/// `.level`, where its library objects are looked for, and the game data it is drawn with.
pub(crate) struct OpenedScene {
  pub source: SceneSource,
  pub objects_roots: XrayRoots,
  pub game: SceneGameData,
}

/// The part files beside a scene, by file name, and the search they are read through.
struct ScenePartFiles<'a> {
  search: &'a XraySearch,
  files: BTreeMap<String, XrayAsset>,
}

/// One tool's part files as read: the parts the editor would load, the files as listed, and why any was left out.
struct SceneToolRead {
  tool: EditableSceneTool,
  description: SceneToolDescription,
  problems: Vec<SceneProblem>,
}

impl OpenedScene {
  /// Reads the scene through `search`, a search of the directory holding its `.level`. Every read, the part directory
  /// listing included, is recorded as the scene's inputs, so a commit to any of them, or a part file added or removed,
  /// reads the scene again.
  ///
  /// # Errors
  ///
  /// Returns an error when its source names no `.level`, the `.level` is not found or does not read as a scene's, its
  /// part directory cannot be listed, or a part carries another scene's GUID
  /// ([`EditableLevelScene::read_from_source`]).
  /// A part file that does not read, or one the editor would not load as its tool's, is kept as a problem on its tool.
  pub(crate) fn read(self, search: XraySearch) -> TauriResult<SessionDerived<SelectedScene>> {
    profiling::scope!("scene open");

    let Self {
      source,
      objects_roots,
      game,
    } = self;
    let name: String = source.require_name()?;
    let roots: XrayRoots = source.get_roots()?;
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let search: XraySearch = search.with_recording(&inputs);
    let level: EditableLevelFile = search
      .read(&format!("{name}.{}", EditableLevelFile::EXTENSION))
      .and_then(|bytes| EditableLevelFile::read_from_bytes(&bytes))
      .map_err(|error| format!("Failed to read scene '{}': {error}", source.path))?;
    let guid: EditableLevelGuid = level
      .get_guid()
      .map_err(|error| format!("Scene '{}' carries no GUID: {error}", source.path))?;
    let files: ScenePartFiles<'_> = ScenePartFiles {
      search: &search,
      files: ScenePartFiles::list(&search, &name)
        .map_err(|error| format!("Failed to list the parts of scene '{}': {error}", source.path))?,
    };
    let EditableSceneRead {
      scene: mut read_scene,
      files: read_files,
    } = EditableLevelScene::read_from_source::<XRayByteOrder>(level, &files)
      .map_err(|error| format!("Failed to read scene '{}': {error}", source.path))?;
    let mut scene: EditableLevelScene = EditableLevelScene {
      level: read_scene.level,
      tools: Vec::new(),
    };
    let mut tools: Vec<SceneToolDescription> = Vec::new();
    let mut problems: Vec<SceneProblem> = Vec::new();

    for tool_name in Self::list_tool_names(&read_files) {
      let tool: Option<EditableSceneTool> = read_scene
        .tools
        .iter()
        .position(|tool| tool.name == tool_name)
        .map(|index| read_scene.tools.remove(index));
      let mut read: SceneToolRead = SceneToolRead::new(tool_name, tool, &read_files);

      if !read.tool.parts.is_empty() {
        scene.tools.push(read.tool);
      }

      tools.push(read.description);
      problems.append(&mut read.problems);
    }

    Ok(SessionDerived::new(
      SelectedScene {
        source,
        name,
        roots,
        objects_roots,
        game,
        guid,
        scene,
        tools,
        problems,
      },
      inputs,
    ))
  }

  /// Reads the open scene again from its source, with the same objects roots and game data.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], or when the scene's roots cannot be searched.
  pub(crate) fn derive(
    current: &SessionSnapshot<SelectedScene>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<SelectedScene>> {
    Self {
      source: current.source.clone(),
      objects_roots: current.objects_roots.clone(),
      game: current.game.clone(),
    }
    .read(assets.search(&current.roots)?)
  }

  /// The tools a read reached a file of, in the editor's order, then the tools it does not save, by name.
  fn list_tool_names(files: &[EditableScenePartFile]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for file in files {
      if !names.contains(&file.tool) {
        names.push(file.tool.clone());
      }
    }

    // Stable over the read's own order, which is by name.
    names.sort_by_key(|name| {
      let kind: Option<SceneToolKind> = SceneToolKind::of(name);

      (kind.is_none(), kind)
    });

    names
  }
}

impl EditableScenePartSource for ScenePartFiles<'_> {
  fn list_files(&self) -> XrfResult<Vec<String>> {
    Ok(self.files.keys().cloned().collect())
  }

  fn read_file(&self, name: &str) -> XrfResult<Vec<u8>> {
    let asset: &XrayAsset = self
      .files
      .get(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Part file '{name}' is not listed")))?;

    self.search.read_asset(asset)
  }
}

impl ScenePartFiles<'_> {
  /// The files of the scene's part directory the editor reads as parts, by name; none where it has no such directory.
  fn list(search: &XraySearch, name: &str) -> XrfResult<BTreeMap<String, XrayAsset>> {
    Ok(
      search
        .list_children(name)?
        .files
        .into_iter()
        .map(|asset| (asset.get_logical_path().file_name().to_owned(), asset))
        .filter(|(file, _)| EditableLevelPart::is_part_file_name(file))
        .collect(),
    )
  }
}

impl SceneToolRead {
  /// Describes the tool's files, each that did not read as a problem, and keeps its parts up to the first the editor
  /// would not load as the tool's (another class or version), since later parts would no longer match their files.
  fn new(name: String, tool: Option<EditableSceneTool>, files: &[EditableScenePartFile]) -> Self {
    let tool_kind: Option<SceneToolKind> = SceneToolKind::of(&name);
    let mut tool: EditableSceneTool = tool.unwrap_or_else(|| EditableSceneTool {
      name: name.clone(),
      parts: Vec::new(),
    });
    let mut description: SceneToolDescription = SceneToolDescription {
      name,
      kind: tool_kind,
      format: tool_kind
        .map(SceneToolKind::get_format)
        .or_else(|| tool.parts.first().map(SceneToolFormat::from)),
      size: 0,
      files: Vec::new(),
    };
    let mut problems: Vec<SceneProblem> = Vec::new();

    for file in files.iter().filter(|file| file.tool == tool.name) {
      description.size += file.size;
      description.files.push(ScenePartFileDescription {
        name: file.name.clone(),
        size: file.size,
        guid: file.guid.as_ref().map(SelectedScene::format_guid),
      });

      if let Some(problem) = &file.problem {
        problems.push(SceneProblem {
          kind: SceneProblemKind::Unreadable,
          tool: tool.name.clone(),
          file: Some(file.name.clone()),
          message: format!("Part does not read: {problem}"),
        });
      }
    }

    if let Some(tool_kind) = tool_kind
      && let Some((index, problem)) = tool
        .parts
        .iter()
        .enumerate()
        .find_map(|(index, part)| Some((index, tool_kind.check(&tool.get_file_name(index), part)?)))
    {
      problems.push(problem);
      tool.parts.truncate(index);
    }

    Self {
      tool,
      description,
      problems,
    }
  }
}
