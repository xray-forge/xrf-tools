use std::collections::HashSet;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostFileName, format_path};

use crate::level::editable_level_file::EditableLevelFile;
use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_level_part::EditableLevelPart;
use crate::scene::editable_scene_part_directory::EditableScenePartDirectory;
use crate::scene::editable_scene_part_file::EditableScenePartFile;
use crate::scene::editable_scene_part_source::EditableScenePartSource;
use crate::scene::editable_scene_read::EditableSceneRead;
use crate::scene::editable_scene_tool::EditableSceneTool;

/// A whole scene as the Level Editor opens it: the `.level` and, in the directory beside it named after it, each
/// tool's part files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableLevelScene {
  pub level: EditableLevelFile,
  /// The tools whose parts the scene holds, ordered by name.
  pub tools: Vec<EditableSceneTool>,
}

impl EditableLevelScene {
  /// The directory a scene's parts live in: the `.level` path without its extension (`EScene::LevelPartPath`).
  pub fn get_parts_directory(level_path: &Path) -> PathBuf {
    level_path.with_extension("")
  }

  /// Reads a scene from the host: the `.level`, then its part directory as [`Self::read_from_source`] reads it.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::read_from_source`] does, or when the `.level` or any part does not read.
  pub fn read_from_path<P: AsRef<Path>>(level_path: &P) -> XrfResult<Self> {
    let directory: EditableScenePartDirectory = EditableScenePartDirectory {
      directory: Self::get_parts_directory(level_path.as_ref()),
    };
    let read: EditableSceneRead = Self::read_from_source(EditableLevelFile::read_from_path(level_path)?, &directory)?;

    if let Some(file) = read.get_problems().next() {
      return Err(XrfError::new_parsing_error(format!(
        "Level part {} does not read: {}",
        format_path(&directory.directory.join(&file.name)),
        file.problem.as_deref().unwrap_or_default()
      )));
    }

    Ok(read.scene)
  }

  /// Reads a scene's parts from `source` as `EScene::LoadLevelPartLTX` finds them: for each `<tool>.part`, that file
  /// and the numbered ones following it without a gap. Other files, such as a `.~part` copy, are not part of the scene.
  ///
  /// A file that does not read as a part with a GUID stops its tool's walk there, so each tool's parts keep their
  /// files' places; the read keeps the file beside the scene with the reason.
  ///
  /// # Errors
  ///
  /// Returns an error when `level` carries no GUID, the directory cannot be listed, or a part carries another GUID: the
  /// editor would skip that stale part, so the scene is refused rather than opened without the tool's objects.
  pub fn read_from_source(
    level: EditableLevelFile,
    source: &impl EditableScenePartSource,
  ) -> XrfResult<EditableSceneRead> {
    let guid: EditableLevelGuid = level.get_guid()?;
    let listed: HashSet<String> = source
      .list_files()?
      .into_iter()
      .filter(|name| EditableLevelPart::is_part_file_name(name))
      .collect();
    let suffix: String = format!(".{}", EditableLevelPart::EXTENSION);
    let mut names: Vec<&str> = listed.iter().filter_map(|name| name.strip_suffix(&suffix)).collect();
    let mut tools: Vec<EditableSceneTool> = Vec::new();
    let mut files: Vec<EditableScenePartFile> = Vec::new();

    names.sort_unstable();

    for name in names {
      let mut tool: EditableSceneTool = EditableSceneTool {
        name: name.to_owned(),
        parts: Vec::new(),
      };
      let class_id: Option<u32> = EditableBinaryPart::get_tool_class_id(name);

      for index in 0.. {
        let file_name: String = tool.get_file_name(index);

        if !listed.contains(&file_name) {
          break;
        }

        let mut file: EditableScenePartFile = EditableScenePartFile {
          tool: tool.name.clone(),
          index,
          name: file_name,
          size: 0,
          guid: None,
          problem: None,
        };
        let (part, found) = match Self::read_part(source, &mut file, class_id) {
          Ok(read) => read,
          Err(error) => {
            file.problem = Some(error.to_string());
            files.push(file);
            break;
          }
        };

        if found != guid {
          return Err(XrfError::new_parsing_error(format!(
            "Level part {} carries GUID {}:{}, not its scene's {}:{}",
            file.name, found.g0, found.g1, guid.g0, guid.g1
          )));
        }

        file.guid = Some(found);
        files.push(file);
        tool.parts.push(part);
      }

      if !tool.parts.is_empty() {
        tools.push(tool);
      }
    }

    Ok(EditableSceneRead {
      scene: Self { level, tools },
      files,
    })
  }

  /// Reads `file` from `source` as a part, its data chunk `class_id`'s, recording its size, and the GUID it carries.
  fn read_part(
    source: &impl EditableScenePartSource,
    file: &mut EditableScenePartFile,
    class_id: Option<u32>,
  ) -> XrfResult<(EditableLevelPart, EditableLevelGuid)> {
    let bytes: Vec<u8> = source.read_file(&file.name)?;

    file.size = bytes.len() as u64;

    let part: EditableLevelPart = EditableLevelPart::read_from_bytes(bytes, class_id)?;
    let guid: EditableLevelGuid = part.get_guid()?;

    Ok((part, guid))
  }

  /// Writes the `.level` and every tool's parts into the directory beside it, leaving any other file there as it is.
  ///
  /// # Errors
  ///
  /// Returns an error, before writing anything, as [`Self::check_writable`] does, or when a file cannot be written.
  pub fn write_to_path<P: AsRef<Path>>(&self, level_path: &P) -> XrfResult {
    self.check_writable(level_path.as_ref())?;
    self.level.write_to_path(level_path)?;

    for (path, part) in self
      .get_part_paths(level_path.as_ref())?
      .iter()
      .zip(self.tools.iter().flat_map(|tool| &tool.parts))
    {
      part.write_to_path(path)?;
    }

    Ok(())
  }

  /// Checks, writing nothing, that [`Self::write_to_path`] would accept the scene at `level_path`. Names compare
  /// without case, as Windows and the editor compare them.
  ///
  /// # Errors
  ///
  /// Returns an error when a tool's name is not one file name, two tools write the same file, the directory cannot be
  /// listed, or it holds a part file the scene would not write and the editor would read: a tool the scene lacks, or a
  /// numbered file past a tool's last.
  pub fn check_writable(&self, level_path: &Path) -> XrfResult {
    let directory: PathBuf = Self::get_parts_directory(level_path);
    let mut written: HashSet<String> = HashSet::new();

    for path in self.get_part_paths(level_path)? {
      let name: String = path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();

      if !written.insert(name) {
        return Err(XrfError::new_invalid_error(format!(
          "Level scene was not written: two of its tools write {}",
          format_path(&path)
        )));
      }
    }

    match Self::list_part_file_names(&directory)?
      .into_iter()
      .find(|name| !written.contains(&name.to_lowercase()))
    {
      Some(stale) => Err(XrfError::new_invalid_error(format!(
        "Level scene was not written: {} holds {stale}, which the scene does not write and the editor would read",
        format_path(&directory)
      ))),
      None => Ok(()),
    }
  }

  /// The part files a write of the scene at `level_path` writes, in the directory beside it, in the scene's order.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when a tool's name is not one file name: empty, `.`, `..`, or holding a separator or a
  /// colon, which would write outside the part directory.
  pub fn get_part_paths(&self, level_path: &Path) -> XrfResult<Vec<PathBuf>> {
    let directory: PathBuf = Self::get_parts_directory(level_path);
    let mut paths: Vec<PathBuf> = Vec::new();

    for tool in &self.tools {
      HostFileName::new(&tool.name, "Scene tool name")?;

      for index in 0..tool.parts.len() {
        paths.push(directory.join(tool.get_file_name(index)));
      }
    }

    Ok(paths)
  }

  pub fn get_tool(&self, name: &str) -> Option<&EditableSceneTool> {
    self.tools.iter().find(|tool| tool.name == name)
  }

  /// The files in the directory the editor reads as parts, sorted; none when the directory does not exist.
  fn list_part_file_names(directory: &Path) -> XrfResult<Vec<String>> {
    let mut names: Vec<String> = EditableScenePartDirectory {
      directory: directory.to_path_buf(),
    }
    .list_files()?
    .into_iter()
    .filter(|name| EditableLevelPart::is_part_file_name(name))
    .collect();

    names.sort();

    Ok(names)
  }
}
