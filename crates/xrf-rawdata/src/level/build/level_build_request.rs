use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_utils::OutputFiles;

use crate::level::build::level_build_export::LevelBuildExport;
use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;
use crate::level::level_scene_exporter::LevelSceneExporter;
use crate::level::parts::level_breakable_conversion::LevelBreakableConversion;

/// A scene's build into a level directory, as a front-end asks for it: the game files always, and the level compiler's
/// files (`build.prj`, the occluders, `level.ltx`, details, AI map, impostors, breakables) where the game material
/// library is named, the objects read from the objects roots ([`Self::get_objects_roots`]).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildRequest {
  /// The scene's main file, `<name>.level`, its parts in the directory beside it named after it.
  pub source: PathBuf,
  /// The level directory the files are written into.
  pub destination: PathBuf,
  /// The objects roots (`$objects$`) the scene's objects are read from, each from the first holding it; none for
  /// `<rawdata>/objects` beside the scene, where a decompile writes them.
  pub objects: Vec<PathBuf>,
  /// The game material library, `gamemtl.xr`; the level compiler's files are written only where it is named.
  pub game_materials: Option<PathBuf>,
  /// The textures root (`$game_textures$`) the details' textures and the impostors' strips are read from; none for
  /// where a decompile writes them ([`Self::get_textures_root`]).
  pub textures: Option<PathBuf>,
  /// How large the impostor atlas may grow.
  pub impostor_limit: LevelBuildImpostorLimit,
  /// Whether the build replaces what it writes where the directory holds it, and removes the files of its own an
  /// earlier build wrote that this one does not; unforced, it refuses before reading the scene.
  pub is_forced: bool,
}

impl LevelBuildRequest {
  /// The directory a rawdata tree keeps objects in, beside the `levels` directory holding its scenes.
  pub const OBJECTS_DIRECTORY: &'static str = "objects";

  /// The files the build owns in its destination: the game files always ([`LevelSceneExporter::FILE_NAMES`]), and
  /// where the game materials are named the level compiler's ([`LevelBuildExport::FILE_NAMES`]) and every breakable
  /// part under `meshes`.
  pub fn get_outputs(&self) -> OutputFiles {
    let outputs: OutputFiles = OutputFiles::new(&self.destination).with_files(LevelSceneExporter::FILE_NAMES);

    match self.game_materials {
      Some(_) => outputs
        .with_files(LevelBuildExport::FILE_NAMES)
        .with_family(LevelBreakableConversion::FAMILY),
      None => outputs,
    }
  }

  /// The objects roots the scene's objects are read from: the ones named, else `<rawdata>/objects` beside the scene's
  /// `levels` directory where it exists, as the SDK's `fs.ltx` puts `$maps$` and `$objects$` side by side and a
  /// decompile writes the scene's objects and the library objects it places there; none otherwise.
  pub fn get_objects_roots(&self) -> Vec<PathBuf> {
    if !self.objects.is_empty() {
      return self.objects.clone();
    }

    self
      .get_rawdata()
      .map(|rawdata| rawdata.join(Self::OBJECTS_DIRECTORY))
      .filter(|objects| objects.is_dir())
      .into_iter()
      .collect()
  }

  /// The textures root the details' textures and the impostors' strips are read from: the one named, else
  /// `<rawdata>/../gamedata/textures`, where a decompile of the scene writes them, as the SDK's `fs.ltx` puts
  /// `$game_textures$` under the gamedata beside its rawdata.
  pub fn get_textures_root(&self) -> PathBuf {
    self.textures.clone().unwrap_or_else(|| {
      self
        .get_rawdata()
        .map_or_else(|| self.source.join("..").join(".."), Path::to_path_buf)
        .join("..")
        .join("gamedata")
        .join("textures")
    })
  }

  /// The rawdata directory holding the scene's `levels` directory, where the scene's path names one.
  fn get_rawdata(&self) -> Option<&Path> {
    self
      .source
      .parent()
      .and_then(Path::parent)
      .filter(|rawdata| !rawdata.as_os_str().is_empty())
  }
}
