use std::collections::BTreeMap;

use byteorder::ByteOrder;
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;
use xrf_level::{BuildAimapFile, BuildPrjFile, LevelHomFile, LevelSomFile};
use xrf_spawn::AlifeObject;

use crate::level::build::level_build_breakable_part::LevelBuildBreakablePart;
use crate::level::build::level_build_census::LevelBuildCensus;
use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::build::level_build_impostor_atlas::LevelBuildImpostorAtlas;
use crate::level::parts::level_map_conversion::LevelMapConversion;
use crate::level::trees::level_tree_impostor_atlas::LevelTreeImpostorAtlas;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// A scene exported for the level compiler: `build.prj`, the occluders and the map's settings written beside it, and
/// what went into them.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelBuildExport {
  pub project: BuildPrjFile,
  /// `level.hom`, where an object occludes the renderer's view.
  pub hom: Option<LevelHomFile>,
  /// `level.som`, where an object occludes sound.
  pub som: Option<LevelSomFile>,
  /// `level.ltx` as the build writes it.
  pub map: Vec<u8>,
  /// `build.details` and its atlas, where the scene holds details.
  pub details: Option<LevelBuildDetails>,
  /// `build.aimap`, where the scene holds AI map nodes.
  pub ai_map: Option<BuildAimapFile>,
  /// `level_lods.dds` and `level_lods_nm.dds`, where a multi-use object has an impostor.
  pub impostors: Option<LevelBuildImpostorAtlas>,
  /// Each breakable part's rigid skeleton, which [`Self::into_files`] writes as `meshes/brkbl#<n>.ogf`.
  pub(crate) breakable_parts: Vec<LevelBuildBreakablePart>,
  /// The entities the build cuts from the geometry, the breakables (`breakable_object`) and then the ladders
  /// (`climable_object`), which `level.spawn` opens with: what
  /// [`LevelSceneExporter::export`](crate::parts::LevelSceneExporter::export) takes as built.
  pub entities: Vec<AlifeObject>,
  /// The parts the build made no entity of, and why, as it reports them and goes on.
  pub refused: Vec<LevelSceneRefusal>,
  pub census: LevelBuildCensus,
}

impl LevelBuildExport {
  /// Every file [`Self::into_files`] may give, sorted by name.
  pub const FILE_NAMES: [&'static str; 9] = [
    BuildAimapFile::FILE_NAME,
    LevelBuildDetails::FILE_NAME,
    BuildPrjFile::FILE_NAME,
    LevelBuildDetails::ATLAS_FILE_NAME,
    LevelHomFile::FILE_NAME,
    LevelMapConversion::FILE_NAME,
    LevelSomFile::FILE_NAME,
    LevelTreeImpostorAtlas::COLOR_FILE_NAME,
    LevelTreeImpostorAtlas::NORMAL_FILE_NAME,
  ];

  /// Each file by its path in the level's directory, written in `T`'s byte order: `build.prj` and `level.ltx` always,
  /// `level.hom` and `level.som` where their occluders are, `build.details` and its atlas where the details are,
  /// `build.aimap` where the AI map is, `level_lods.dds` and `level_lods_nm.dds` where the impostors are, and each
  /// breakable part under `meshes`.
  ///
  /// # Errors
  ///
  /// Returns an error when a file does not write.
  pub fn into_files<T: ByteOrder>(self) -> XrfResult<BTreeMap<String, Vec<u8>>> {
    let encode = |write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult<Vec<u8>> {
      let mut writer: ChunkWriter = ChunkWriter::new();

      write(&mut writer)?;

      Ok(writer.buffer)
    };
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut insert = |name: &str, bytes: Vec<u8>| files.insert(name.to_owned(), bytes);

    insert(
      BuildPrjFile::FILE_NAME,
      encode(&|writer| self.project.write::<T>(writer))?,
    );
    insert(LevelMapConversion::FILE_NAME, self.map);

    if let Some(hom) = &self.hom {
      insert(LevelHomFile::FILE_NAME, encode(&|writer| hom.write::<T>(writer))?);
    }

    if let Some(som) = &self.som {
      insert(LevelSomFile::FILE_NAME, encode(&|writer| som.write::<T>(writer))?);
    }

    if let Some(ai_map) = &self.ai_map {
      insert(BuildAimapFile::FILE_NAME, encode(&|writer| ai_map.write::<T>(writer))?);
    }

    if let Some(details) = self.details {
      insert(
        LevelBuildDetails::FILE_NAME,
        encode(&|writer| details.file.write::<T>(writer))?,
      );
      insert(LevelBuildDetails::ATLAS_FILE_NAME, details.atlas);
    }

    if let Some(impostors) = self.impostors {
      insert(LevelTreeImpostorAtlas::COLOR_FILE_NAME, impostors.color);
      insert(LevelTreeImpostorAtlas::NORMAL_FILE_NAME, impostors.normal);
    }

    for part in &self.breakable_parts {
      insert(&part.path, part.write::<T>()?);
    }

    Ok(files)
  }
}
