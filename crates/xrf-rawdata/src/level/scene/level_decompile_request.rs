use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_editable_level::EditableLevelFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::level_surface_inference::{LevelSurfaceInference, LevelSurfaces};
use crate::level::scene::level_decompile_options::LevelDecompileOptions;

/// A level's decompile into a scene on disk, as a front-end asks for it: the paths, and the options it may leave to
/// their defaults, which [`Self::get_options`] and [`Self::get_objects_root`] resolve as the SDK lays a level and its
/// rawdata out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileRequest {
  /// The compiled level's directory, `<gamedata>/levels/<level>`.
  pub source: PathBuf,
  /// The scene's main file, `<rawdata>/levels/<name>.level`; its name is the level's.
  pub destination: PathBuf,
  /// The objects root the scene's objects are written under; `<rawdata>/objects` beside the scene by default.
  pub objects: Option<PathBuf>,
  /// The game textures root the objects' textures are written under; `<rawdata>/../gamedata/textures` by default.
  pub textures: Option<PathBuf>,
  /// The root spawn whose objects stand in for `level.spawn`'s entities, by name in the level's `gamedata/spawns`;
  /// `all` where that exists by default.
  pub root_spawn: Option<String>,
  pub surfaces: LevelSurfaces,
  /// The game material library; the level's gamedata's `gamemtl.xr` where it exists by default.
  pub game_materials: Option<PathBuf>,
  /// The compiler shader library; the level's gamedata's `shaders_xrlc.xr` where it exists by default.
  pub compiler_shaders: Option<PathBuf>,
  /// Object library roots whose surfaces break ties between compiler shaders, such as an SDK's `rawdata/objects`.
  pub libraries: Vec<PathBuf>,
  /// Whether an existing scene, its objects and textures are overwritten rather than refused.
  pub is_forced: bool,
}

impl LevelDecompileRequest {
  /// The root spawn a new game starts from, which a decompile reads by default.
  pub const DEFAULT_ROOT_SPAWN: &'static str = "all";

  /// The decompile options of this request by `tool` at `tool_version`, every default resolved.
  ///
  /// # Errors
  ///
  /// Returns an error when the destination names no `.level` file, or a library or root spawn named does not exist.
  pub fn get_options(&self, tool: &str, tool_version: &str) -> XrfResult<LevelDecompileOptions> {
    let gamedata: PathBuf = self.get_gamedata();
    // A library named must exist; one by default is taken where the level's gamedata holds it.
    let library = |named: &Option<PathBuf>, file: &str| -> XrfResult<Option<PathBuf>> {
      match named {
        Some(path) if path.is_file() => Ok(Some(path.clone())),
        Some(path) => Err(XrfError::new_not_found_error(format!(
          "{} does not exist",
          format_path(path)
        ))),
        None => Ok(Some(gamedata.join(file)).filter(|path| path.is_file())),
      }
    };

    Ok(LevelDecompileOptions {
      name: self.get_level_name()?,
      tool: tool.to_owned(),
      tool_version: tool_version.to_owned(),
      root_spawn: self.get_root_spawn(&gamedata)?,
      surfaces: self.surfaces,
      game_materials: library(&self.game_materials, LevelSurfaceInference::GAME_MATERIALS_FILE)?,
      compiler_shaders: library(&self.compiler_shaders, LevelSurfaceInference::COMPILER_SHADERS_FILE)?,
      libraries: self.libraries.clone(),
    })
  }

  /// The objects root the scene's objects go under: the one named, else `<rawdata>/objects` beside the scene, as the
  /// SDK's `fs.ltx` puts `$maps$` and `$objects$` side by side under its rawdata.
  pub fn get_objects_root(&self) -> PathBuf {
    self
      .objects
      .clone()
      .unwrap_or_else(|| self.get_rawdata().join("objects"))
  }

  /// The game textures root the objects' textures go under: the one named, else `<rawdata>/../gamedata/textures`, as
  /// the SDK's `fs.ltx` puts `$game_textures$` under the gamedata beside its rawdata.
  pub fn get_textures_root(&self) -> PathBuf {
    self
      .textures
      .clone()
      .unwrap_or_else(|| self.get_rawdata().join("..").join("gamedata").join("textures"))
  }

  /// The level's name, the destination's file stem.
  ///
  /// # Errors
  ///
  /// Returns an error when the destination does not end in `.level`.
  pub fn get_level_name(&self) -> XrfResult<String> {
    self
      .destination
      .extension()
      .filter(|extension| extension.eq_ignore_ascii_case(EditableLevelFile::EXTENSION))
      .and(self.destination.file_stem())
      .and_then(|stem| stem.to_str())
      .filter(|stem| !stem.is_empty())
      .map(str::to_owned)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "{} does not name a .{} file",
          format_path(&self.destination),
          EditableLevelFile::EXTENSION
        ))
      })
  }

  /// The `gamedata` directory holding `levels\<level>`, named by the source's parents where it has them.
  fn get_gamedata(&self) -> PathBuf {
    match self.source.parent().and_then(Path::parent) {
      Some(gamedata) if !gamedata.as_os_str().is_empty() => gamedata.to_path_buf(),
      _ => self.source.join("..").join(".."),
    }
  }

  /// The root spawn named in `gamedata`, `spawns/<name>.spawn`. Without a name, `all.spawn` where it exists, else none.
  fn get_root_spawn(&self, gamedata: &Path) -> XrfResult<Option<PathBuf>> {
    let name: Option<&str> = self.root_spawn.as_deref();
    let path: PathBuf = gamedata
      .join("spawns")
      .join(format!("{}.spawn", name.unwrap_or(Self::DEFAULT_ROOT_SPAWN)));

    match (path.is_file(), name) {
      (true, _) => Ok(Some(path)),
      (false, None) => Ok(None),
      (false, Some(_)) => Err(XrfError::new_not_found_error(format!(
        "{} does not exist",
        format_path(&path)
      ))),
    }
  }

  /// The rawdata directory holding the scene's `levels` directory.
  fn get_rawdata(&self) -> PathBuf {
    let named = |path: &&Path| !path.as_os_str().is_empty();

    match self.destination.parent().filter(named) {
      Some(levels) => match levels.parent().filter(named) {
        Some(rawdata) => rawdata.to_path_buf(),
        None => levels.join(".."),
      },
      None => PathBuf::from(".."),
    }
  }
}
