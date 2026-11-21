use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::format_path;

use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::surfaces::level_surfaces::LevelSurfaces;
use crate::provenance::level_scene_libraries::LevelSceneLibraries;
use crate::provenance::level_scene_source::LevelSceneSource;
use crate::provenance::sidecar::Sidecar;

/// The game material and compiler shader libraries a level's geometry is decompiled with, and the sidecar's record of
/// each read, whatever dresses the surfaces.
pub(crate) struct LevelGeometryLibraries {
  /// Both, where the surfaces are inferred from them; they then also tell the placed library objects apart.
  pub(crate) inferred: Option<(GameMtlFile, ShaderCompilerFile)>,
  /// Where the surfaces are not inferred and library roots are named: the game materials telling the placed objects'
  /// ladders and breakables, and the compiler shaders naming their surfaces, each where the options name it.
  pub(crate) game_materials: Option<GameMtlFile>,
  compiler_shaders: Option<ShaderCompilerFile>,
  pub(crate) record: LevelSceneLibraries,
}

impl LevelGeometryLibraries {
  /// The libraries `options` name that the geometry of a level reads, `has_collision` saying whether it holds the
  /// collision form inferring surfaces needs.
  ///
  /// # Errors
  ///
  /// Returns an error when inferring surfaces lacks a library it needs, or a library named does not read.
  pub(crate) fn read(options: &LevelDecompileOptions, has_collision: bool) -> XrfResult<Self> {
    if options.surfaces == LevelSurfaces::Infer && has_collision {
      return Self::read_inferred(options);
    }

    let is_placing: bool = !options.libraries.is_empty();
    let mut record: LevelSceneLibraries = LevelSceneLibraries::default();
    let game_materials: Option<GameMtlFile> = match (&options.game_materials, is_placing) {
      (Some(path), true) => {
        let (bytes, source) = Self::read_library(path)?;

        record.game_materials = Some(source);
        Some(GameMtlFile::read_from_bytes(bytes)?)
      }
      _ => None,
    };
    let compiler_shaders: Option<ShaderCompilerFile> = match (&options.compiler_shaders, is_placing) {
      (Some(path), true) => {
        let (bytes, source) = Self::read_library(path)?;

        record.compiler_shaders = Some(source);
        Some(ShaderCompilerFile::read_from_bytes(bytes)?)
      }
      _ => None,
    };

    Ok(Self {
      inferred: None,
      game_materials,
      compiler_shaders,
      record,
    })
  }

  /// The compiler shaders the placed library objects' surfaces are named from: the inference's, else those named.
  pub(crate) fn get_compiler_shaders(&self) -> Option<&ShaderCompilerFile> {
    self
      .inferred
      .as_ref()
      .map(|(_, shaders)| shaders)
      .or(self.compiler_shaders.as_ref())
  }

  /// What dresses the surfaces beside the library's pairings: the game material and compiler shader libraries the
  /// options name, both needed.
  fn read_inferred(options: &LevelDecompileOptions) -> XrfResult<Self> {
    let read = |path: Option<&PathBuf>, what: &str| -> XrfResult<(Vec<u8>, LevelSceneSource)> {
      Self::read_library(path.ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Inferring surfaces needs the {what}; name it, or dress surfaces with the editor's defaults"
        ))
      })?)
    };
    let (game_materials, game_materials_source) =
      read(options.game_materials.as_ref(), "game material library (gamemtl.xr)")?;
    let (compiler_shaders, compiler_shaders_source) = read(
      options.compiler_shaders.as_ref(),
      "compiler shader library (shaders_xrlc.xr)",
    )?;

    Ok(Self {
      inferred: Some((
        GameMtlFile::read_from_bytes(game_materials)?,
        ShaderCompilerFile::read_from_bytes(compiler_shaders)?,
      )),
      game_materials: None,
      compiler_shaders: None,
      record: LevelSceneLibraries {
        game_materials: Some(game_materials_source),
        compiler_shaders: Some(compiler_shaders_source),
      },
    })
  }

  /// A library file's bytes and its record, named and hashed for the sidecar.
  fn read_library(path: &Path) -> XrfResult<(Vec<u8>, LevelSceneSource)> {
    let bytes: Vec<u8> = std::fs::read(path)
      .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))?;
    let source: LevelSceneSource = LevelSceneSource {
      file: format_path(path).to_string(),
      sha256: Sidecar::get_sha256_hex(&bytes),
    };

    Ok((bytes, source))
  }
}
