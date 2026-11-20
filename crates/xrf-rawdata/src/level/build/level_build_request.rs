use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_utils::OutputFiles;

use crate::level::build::level_build_export::LevelBuildExport;
use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;
use crate::level::level_scene_exporter::LevelSceneExporter;
use crate::level::parts::level_breakable_conversion::LevelBreakableConversion;

/// A front-end's request to build a scene into a level directory: the game files always, and the level compiler's
/// files where the game material library is named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildRequest {
  /// The scene's main file, `<name>.level`, its parts in the directory beside it named after it.
  pub source: PathBuf,
  /// The level directory the files are written into.
  pub destination: PathBuf,
  /// The objects roots (`$objects$`), each object read from the first holding it; empty for the default
  /// ([`Self::get_objects_roots`]).
  pub objects: Vec<PathBuf>,
  /// The game material library, `gamemtl.xr`; the level compiler's files are written only where it is named.
  pub game_materials: Option<PathBuf>,
  /// The textures root (`$game_textures$`) of the details' textures and the impostors' strips; none for the default
  /// ([`Self::get_textures_root`]).
  pub textures: Option<PathBuf>,
  pub impostor_limit: LevelBuildImpostorLimit,
  /// Whether the build replaces files the directory holds and removes its own stale ones; unforced, it refuses a
  /// directory holding a file it owns before reading the scene.
  pub is_forced: bool,
}

impl LevelBuildRequest {
  /// The directory a rawdata tree keeps objects in, beside the `levels` directory holding its scenes.
  pub const OBJECTS_DIRECTORY: &'static str = "objects";

  /// The files the build owns in its destination: the game files ([`LevelSceneExporter::FILE_NAMES`]), and where the
  /// game materials are named the level compiler's ([`LevelBuildExport::FILE_NAMES`]) and the breakable parts.
  pub fn get_outputs(&self) -> OutputFiles {
    let outputs: OutputFiles = OutputFiles::new(&self.destination).with_files(LevelSceneExporter::FILE_NAMES);

    match self.game_materials {
      Some(_) => outputs
        .with_files(LevelBuildExport::FILE_NAMES)
        .with_family(LevelBreakableConversion::FAMILY),
      None => outputs,
    }
  }

  /// The objects roots: the ones named, else `<rawdata>/objects` beside the scene's `levels` directory where it
  /// exists, as the SDK's `fs.ltx` lays them out and a decompile writes them; none otherwise.
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

  /// The textures root: the one named, else `<rawdata>/../gamedata/textures`, as the SDK's `fs.ltx` lays it out and a
  /// decompile writes it.
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
