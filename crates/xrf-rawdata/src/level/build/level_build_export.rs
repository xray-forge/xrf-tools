use std::collections::BTreeMap;

use byteorder::ByteOrder;
use serde::Serialize;
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;
use xrf_level::{BuildAimapFile, BuildPrjFile, LevelHomFile, LevelSomFile};
use xrf_spawn::AlifeObject;

use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::build::level_build_impostors::LevelBuildImpostorAtlas;
use crate::level::level_map_conversion::LevelMapConversion;
use crate::level::level_tree_impostor::LevelTreeImpostorAtlas;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

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
  /// The entities the build cuts from the geometry, the ladders (`climable_object`), which `level.spawn` opens with:
  /// what [`LevelSceneExporter::export`](crate::LevelSceneExporter::export) takes as built.
  pub entities: Vec<AlifeObject>,
  /// The parts the build made no entity of, and why, as it reports them and goes on.
  pub refused: Vec<LevelSceneRefusal>,
  pub census: LevelBuildCensus,
}

/// What a build took from the scene and left out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildCensus {
  /// Static objects whose meshes went into the geometry.
  pub static_objects: usize,
  /// Objects of no static geometry: dynamic ones, and occluders, which `level.hom` and `level.som` take.
  pub other_objects: usize,
  pub faces: usize,
  /// Faces of occluder, breakable and climbable surfaces, which other exports take.
  pub skipped_faces: usize,
  /// Faces of no area, which the Build drops.
  pub invalid_faces: usize,
  pub static_lights: usize,
  pub dynamic_lights: usize,
  pub glows: usize,
  pub portals: usize,
  pub occluder_polygons: usize,
  pub sound_occluder_polygons: usize,
  /// Multi-use models and the placements referencing them.
  pub multi_use_models: usize,
  pub multi_use_references: usize,
  /// Impostors of those models, whose strips the atlas merges.
  pub impostors: usize,
  /// AI map nodes the map holds.
  pub ai_map_nodes: usize,
  /// Detail objects the slots plant, and the textures their atlas merges.
  pub detail_objects: usize,
  pub detail_textures: usize,
  /// Ladders made of climbable faces, and parts of them refused.
  pub climbable_objects: usize,
  pub refused_parts: usize,
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

  /// Each file by name, written in `T`'s byte order: `build.prj` and `level.ltx` always, `level.hom` and `level.som`
  /// where their occluders are, `build.details` and its atlas where the details are, `build.aimap` where the AI map is,
  /// and `level_lods.dds` and `level_lods_nm.dds` where the impostors are.
  ///
  /// # Errors
  ///
  /// Returns an error when a file does not write.
  pub fn into_files<T: ByteOrder>(self) -> XrfResult<BTreeMap<&'static str, Vec<u8>>> {
    let encode = |write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult<Vec<u8>> {
      let mut writer: ChunkWriter = ChunkWriter::new();

      write(&mut writer)?;

      Ok(writer.buffer)
    };
    let mut files: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();

    files.insert(
      BuildPrjFile::FILE_NAME,
      encode(&|writer| self.project.write::<T>(writer))?,
    );
    files.insert(LevelMapConversion::FILE_NAME, self.map);

    if let Some(hom) = &self.hom {
      files.insert(LevelHomFile::FILE_NAME, encode(&|writer| hom.write::<T>(writer))?);
    }

    if let Some(som) = &self.som {
      files.insert(LevelSomFile::FILE_NAME, encode(&|writer| som.write::<T>(writer))?);
    }

    if let Some(ai_map) = &self.ai_map {
      files.insert(BuildAimapFile::FILE_NAME, encode(&|writer| ai_map.write::<T>(writer))?);
    }

    if let Some(details) = self.details {
      files.insert(
        LevelBuildDetails::FILE_NAME,
        encode(&|writer| details.file.write::<T>(writer))?,
      );
      files.insert(LevelBuildDetails::ATLAS_FILE_NAME, details.atlas);
    }

    if let Some(impostors) = self.impostors {
      files.insert(LevelTreeImpostorAtlas::COLOR_FILE_NAME, impostors.color);
      files.insert(LevelTreeImpostorAtlas::NORMAL_FILE_NAME, impostors.normal);
    }

    Ok(files)
  }
}
