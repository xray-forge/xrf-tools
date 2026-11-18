use std::collections::HashSet;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostFileName, format_path};

use crate::level::editable_level_file::EditableLevelFile;
use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_level_part::EditableLevelPart;
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

  /// Reads a scene: the `.level`, then for every `<tool>.part` in the parts directory that file and the numbered ones
  /// after it, as long as they follow without a gap, the way `EScene::LoadLevelPartLTX` reads them. Any other file in
  /// the directory, such as a `.~part` copy of an earlier save, is not part of the scene.
  ///
  /// # Errors
  ///
  /// Returns an error when the `.level` or a part cannot be read, or a part carries a GUID other than the scene's,
  /// which the editor skips as a stale version of the part.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(level_path: &P) -> XrfResult<Self> {
    let level: EditableLevelFile = EditableLevelFile::read_from_path(level_path)?;
    let guid: EditableLevelGuid = level.get_guid()?;
    let directory: PathBuf = Self::get_parts_directory(level_path.as_ref());
    let mut tools: Vec<EditableSceneTool> = Vec::new();

    for name in Self::list_tool_names(&directory)? {
      let mut tool: EditableSceneTool = EditableSceneTool {
        name,
        parts: Vec::new(),
      };

      loop {
        let path: PathBuf = directory.join(tool.get_file_name(tool.parts.len()));

        if !path.is_file() {
          break;
        }

        let part: EditableLevelPart = EditableLevelPart::read_from_path::<T, _>(&path)?;
        let part_guid: EditableLevelGuid = part.get_guid()?;

        if part_guid != guid {
          return Err(XrfError::new_parsing_error(format!(
            "Level part {} carries GUID {}:{}, not its scene's {}:{}",
            format_path(&path),
            part_guid.g0,
            part_guid.g1,
            guid.g0,
            guid.g1
          )));
        }

        tool.parts.push(part);
      }

      tools.push(tool);
    }

    Ok(Self { level, tools })
  }

  /// Writes the `.level` and every tool's parts into the directory beside it, leaving any other file there as it is.
  ///
  /// # Errors
  ///
  /// Returns an error, before writing anything, when the directory holds a part file the scene would not write: the
  /// editor would read it as part of the scene, a tool the scene no longer has or a numbered file past its last.
  /// Returns an error too when a file cannot be written as its own writer says.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, level_path: &P) -> XrfResult {
    self.check_writable(level_path.as_ref())?;
    self.level.write_to_path(level_path)?;

    for (path, part) in self
      .get_part_paths(level_path.as_ref())?
      .iter()
      .zip(self.tools.iter().flat_map(|tool| &tool.parts))
    {
      part.write_to_path::<T, _>(path)?;
    }

    Ok(())
  }

  /// Checks, writing nothing, that [`Self::write_to_path`] would not refuse the scene at `level_path`: that every tool
  /// names one file, no two of them the same, and the part directory beside it holds no part file the scene would not
  /// write. Names are compared without case, as Windows and the editor compare them.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first such file, which the editor would read as part of the scene: a tool the scene
  /// does not have, or a numbered file past a tool's last. Returns an error too when a tool's name is not one file name,
  /// two tools write the same file, or the directory cannot be listed.
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

  /// The names of the tools with a `<name>.part` in the directory, sorted.
  fn list_tool_names(directory: &Path) -> XrfResult<Vec<String>> {
    let suffix: String = format!(".{}", EditableLevelPart::EXTENSION);

    Ok(
      Self::list_part_file_names(directory)?
        .into_iter()
        .filter_map(|file_name| file_name.strip_suffix(&suffix).map(str::to_owned))
        .collect(),
    )
  }

  /// The files in the directory the editor reads as parts, sorted; none when the directory does not exist.
  fn list_part_file_names(directory: &Path) -> XrfResult<Vec<String>> {
    if !directory.is_dir() {
      return Ok(Vec::new());
    }

    let mut names: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(directory)? {
      let entry = entry?;

      if entry.file_type()?.is_file()
        && let Some(file_name) = entry.file_name().to_str()
        && EditableLevelPart::is_part_file_name(file_name)
      {
        names.push(file_name.to_owned());
      }
    }

    names.sort();

    Ok(names)
  }
}
