use std::collections::BTreeMap;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableLevelFile, EditableLevelGuid, EditableLevelPart, EditableLevelScene, EditableSceneTool,
};
use xrf_error::XrfResult;
use xrf_vfs::{XrayAsset, XrayInputRecord, XrayRoots, XraySearch};

use crate::scene::{
  ScenePartFileDescription, SceneProblem, SceneProblemKind, SceneSource, SceneToolDescription, SceneToolFormat,
  SceneToolKind, SelectedScene,
};

/// How a Level Editor scene is read through the shared asset state: what an open commits, and what a commit's follow
/// derives again once it moved a file the scene was read from.
pub(crate) struct OpenedScene;

/// The part files beside a scene, by file name, and what each is read through and checked against.
struct ScenePartFiles<'a> {
  search: &'a XraySearch,
  files: BTreeMap<String, XrayAsset>,
  guid: EditableLevelGuid,
}

/// One tool's part files as read: the parts the editor would load, the files as listed, and why any was left out.
struct SceneToolRead {
  tool: EditableSceneTool,
  description: SceneToolDescription,
  problems: Vec<SceneProblem>,
}

impl OpenedScene {
  /// Reads the scene `source` names through `search`, a search of the directory holding its `.level`: the `.level`,
  /// the listing of its part directory and each tool's part files as the editor finds them, every read recorded as the
  /// scene's inputs, so a commit to any of them, or a part file come or gone, reads the scene again. Its library objects
  /// are looked for in `objects_roots` where they are described.
  ///
  /// # Errors
  ///
  /// Returns an error when `source` names no `.level`, the `.level` is not found or does not read as a scene's, or its
  /// part directory cannot be listed. A part file that does not read, or one the editor would skip, is kept as a
  /// problem on its tool.
  pub(crate) fn read(
    search: XraySearch,
    source: SceneSource,
    objects_roots: XrayRoots,
  ) -> TauriResult<SessionDerived<SelectedScene>> {
    profiling::scope!("scene open");

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
      guid,
    };
    let mut scene: EditableLevelScene = EditableLevelScene {
      level,
      tools: Vec::new(),
    };
    let mut tools: Vec<SceneToolDescription> = Vec::new();
    let mut problems: Vec<SceneProblem> = Vec::new();

    for tool in files.list_tool_names() {
      let mut read: SceneToolRead = files.read_tool(tool);

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
        guid,
        scene,
        tools,
        problems,
      },
      inputs,
    ))
  }

  /// The open scene read again over the source it was opened from, its library objects looked for where they were.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], or where the scene's roots cannot be searched.
  pub(crate) fn derive(
    current: &SessionSnapshot<SelectedScene>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<SelectedScene>> {
    Self::read(
      assets.search(&current.roots)?,
      current.source.clone(),
      current.objects_roots.clone(),
    )
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

  /// The tools with a first part file, `<tool>.part`, in the editor's order, those it does not save after them by
  /// name.
  fn list_tool_names(&self) -> Vec<String> {
    let suffix: String = format!(".{}", EditableLevelPart::EXTENSION);
    let mut names: Vec<String> = self
      .files
      .keys()
      .filter_map(|file| file.strip_suffix(&suffix).map(str::to_owned))
      .collect();

    // Stable over the files' own order, which is by name.
    names.sort_by_key(|name| {
      let kind: Option<SceneToolKind> = SceneToolKind::of(name);

      (kind.is_none(), kind)
    });

    names
  }

  /// The tool's files as `EScene::LoadLevelPartLTX` reads them: its first, then each numbered one after it until one is
  /// missing, every one that reads, carries the scene's GUID and is the tool's to load kept as a part.
  fn read_tool(&self, name: String) -> SceneToolRead {
    let tool_kind: Option<SceneToolKind> = SceneToolKind::of(&name);
    let mut read: SceneToolRead = SceneToolRead {
      tool: EditableSceneTool {
        name: name.clone(),
        parts: Vec::new(),
      },
      description: SceneToolDescription {
        name,
        kind: tool_kind,
        format: tool_kind.map(SceneToolKind::get_format),
        size: 0,
        files: Vec::new(),
      },
      problems: Vec::new(),
    };

    for index in 0.. {
      let file: String = read.tool.get_file_name(index);
      let Some(asset) = self.files.get(&file) else {
        break;
      };
      let problem = |kind: SceneProblemKind, message: String| SceneProblem {
        kind,
        tool: read.tool.name.clone(),
        file: Some(file.clone()),
        message,
      };
      let bytes: Vec<u8> = match self.search.read_asset(asset) {
        Ok(bytes) => bytes,
        Err(error) => {
          read.problems.push(problem(
            SceneProblemKind::Unreadable,
            format!("Part was not read: {error}"),
          ));
          read.description.files.push(ScenePartFileDescription {
            name: file,
            size: 0,
            guid: None,
          });

          continue;
        }
      };
      let size: u64 = bytes.len() as u64;
      let part: XrfResult<EditableLevelPart> = EditableLevelPart::read_from_bytes::<XRayByteOrder>(bytes);
      let part_guid: Option<EditableLevelGuid> = part.as_ref().ok().and_then(|part| part.get_guid().ok());
      let part_format: Option<SceneToolFormat> = part.as_ref().ok().map(SceneToolFormat::from);
      let accepted: Result<EditableLevelPart, SceneProblem> = part
        .map_err(|error| problem(SceneProblemKind::Unreadable, format!("Part does not read: {error}")))
        .and_then(|part| match part_guid {
          Some(found) if found == self.guid => Ok(part),
          Some(found) => Err(problem(
            SceneProblemKind::ForeignGuid,
            format!(
              "Part carries GUID {}, not its scene's {}",
              SelectedScene::format_guid(&found),
              SelectedScene::format_guid(&self.guid)
            ),
          )),
          None => Err(problem(
            SceneProblemKind::Unreadable,
            String::from("Part carries no GUID"),
          )),
        })
        .and_then(
          |part| match tool_kind.and_then(|tool_kind| tool_kind.check(&file, &part)) {
            Some(problem) => Err(problem),
            None => Ok(part),
          },
        );

      read.description.size += size;
      read.description.format = read.description.format.or(part_format);
      read.description.files.push(ScenePartFileDescription {
        name: file,
        size,
        guid: part_guid.as_ref().map(SelectedScene::format_guid),
      });

      match accepted {
        Ok(part) => read.tool.parts.push(part),
        Err(problem) => read.problems.push(problem),
      }
    }

    read
  }
}
