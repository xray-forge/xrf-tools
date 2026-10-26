use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

use crate::build_prj::build_prj_params::BuildPrjParams;
use crate::build_prj::geometry::build_prj_face::BuildPrjFace;
use crate::build_prj::geometry::build_prj_glow::BuildPrjGlow;
use crate::build_prj::geometry::build_prj_portal::BuildPrjPortal;
use crate::build_prj::lights::build_prj_dynamic_light::BuildPrjDynamicLight;
use crate::build_prj::lights::build_prj_light_control::BuildPrjLightControl;
use crate::build_prj::models::build_prj_lod::BuildPrjLod;
use crate::build_prj::models::build_prj_mu_model::BuildPrjMuModel;
use crate::build_prj::models::build_prj_mu_reference::BuildPrjMuReference;
use crate::build_prj::surfaces::build_prj_material::BuildPrjMaterial;
use crate::build_prj::surfaces::build_prj_shader::BuildPrjShader;
use crate::build_prj::surfaces::build_prj_texture::BuildPrjTexture;
use crate::level::dynamic_lights::level_dynamic_light::LevelDynamicLight;

/// The scene the level editor hands xrLC, `build.prj` (`SceneBuilder::SaveBuild`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjFile {
  /// `XRCL_CURRENT_VERSION`: 17 for Shadow of Chernobyl's editor, 18 from Clear Sky's on.
  pub version: u32,
  pub params: BuildPrjParams,
  /// The static geometry's vertices, in world space.
  pub vertices: Vec<Vector3d<f32>>,
  pub faces: Vec<BuildPrjFace>,
  /// A smoothing group per face, which version 18 added.
  pub smoothing_groups: Option<Vec<u32>>,
  pub materials: Vec<BuildPrjMaterial>,
  pub render_shaders: Vec<BuildPrjShader>,
  pub compiler_shaders: Vec<BuildPrjShader>,
  pub textures: Vec<BuildPrjTexture>,
  pub glows: Vec<BuildPrjGlow>,
  pub portals: Vec<BuildPrjPortal>,
  pub light_controls: Vec<BuildPrjLightControl>,
  /// The lights baked into the lightmaps, each naming its layer by index into the light controls.
  pub static_lights: Vec<LevelDynamicLight>,
  /// The lights the multi-use models are lit by.
  pub dynamic_lights: Vec<BuildPrjDynamicLight>,
  pub lods: Vec<BuildPrjLod>,
  pub mu_models: Vec<BuildPrjMuModel>,
  pub mu_references: Vec<BuildPrjMuReference>,
}

impl BuildPrjFile {
  /// The file the level editor leaves its scene as, in a level's directory.
  pub const FILE_NAME: &'static str = "build.prj";

  /// The version Shadow of Chernobyl's editor writes, without smoothing groups.
  pub const SOC_VERSION: u32 = 17;

  /// The version every later editor writes, and the only one the detail light compiler accepts.
  pub const CURRENT_VERSION: u32 = 18;

  pub const VERSION_CHUNK_ID: u32 = 0;
  pub const PARAMS_CHUNK_ID: u32 = 1;
  pub const VERTICES_CHUNK_ID: u32 = 2;
  pub const FACES_CHUNK_ID: u32 = 3;
  pub const MATERIALS_CHUNK_ID: u32 = 4;
  pub const RENDER_SHADERS_CHUNK_ID: u32 = 5;
  pub const COMPILER_SHADERS_CHUNK_ID: u32 = 6;
  pub const TEXTURES_CHUNK_ID: u32 = 7;
  pub const GLOWS_CHUNK_ID: u32 = 8;
  pub const PORTALS_CHUNK_ID: u32 = 9;
  pub const LIGHT_CONTROLS_CHUNK_ID: u32 = 10;
  pub const STATIC_LIGHTS_CHUNK_ID: u32 = 11;
  pub const DYNAMIC_LIGHTS_CHUNK_ID: u32 = 12;
  pub const LODS_CHUNK_ID: u32 = 13;
  pub const MU_MODELS_CHUNK_ID: u32 = 14;
  pub const MU_REFERENCES_CHUNK_ID: u32 = 15;
  pub const SMOOTHING_GROUPS_CHUNK_ID: u32 = 16;

  /// Bytes one vertex occupies.
  pub const VERTEX_SIZE: u64 = 12;

  /// Reads a build project from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a build project this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Build project was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a build project from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a build project this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a build project this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is unknown, repeated, absent (smoothing groups aside) or not wholly read, or the
  /// version is one no compiler reads.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut ids: HashSet<u32> = HashSet::new();

    for chunk in &chunks {
      if chunk.id > Self::SMOOTHING_GROUPS_CHUNK_ID {
        return Err(XrfError::new_invalid_error(format!(
          "Build project carries chunk {}, which no editor writes",
          chunk.id
        )));
      }

      if !ids.insert(chunk.id) {
        return Err(XrfError::new_invalid_error(format!(
          "Build project carries chunk {} twice, and a rewrite would keep only the first",
          chunk.id
        )));
      }
    }

    let version: u32 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u32_payload::<T>()?;

    if !(Self::SOC_VERSION..=Self::CURRENT_VERSION).contains(&version) {
      return Err(XrfError::new_invalid_error(format!(
        "Build project version {version} is not one an editor writes"
      )));
    }

    let mut params_chunk: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::PARAMS_CHUNK_ID)?;
    let params: BuildPrjParams = params_chunk.read_xr::<T, _>()?;

    params_chunk.assert_read("Expect all data to be read from build project params")?;

    let smoothing_groups: Option<Vec<u32>> =
      match ChunkReader::find_optional_by_id(&chunks, Self::SMOOTHING_GROUPS_CHUNK_ID) {
        Some(mut chunk) => Some(Self::read_smoothing_groups::<T, D>(&mut chunk)?),
        None => None,
      };

    let mut light_controls_chunk: ChunkReader<D> =
      ChunkReader::find_required_by_id(&chunks, Self::LIGHT_CONTROLS_CHUNK_ID)?;
    let mut light_controls: Vec<BuildPrjLightControl> = Vec::new();

    while !light_controls_chunk.is_ended() {
      light_controls.push(light_controls_chunk.read_xr::<T, _>()?);
    }

    let mut mu_models_chunk: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::MU_MODELS_CHUNK_ID)?;
    let mut mu_models: Vec<BuildPrjMuModel> = Vec::new();

    while !mu_models_chunk.is_ended() {
      mu_models.push(BuildPrjMuModel::read::<T, _>(
        &mut mu_models_chunk,
        Self::has_smoothing_groups(version),
      )?);
    }

    Ok(Self {
      version,
      params,
      vertices: Self::read_records::<T, D, _>(&chunks, Self::VERTICES_CHUNK_ID, Self::VERTEX_SIZE, "vertices")?,
      faces: Self::read_records::<T, D, _>(&chunks, Self::FACES_CHUNK_ID, BuildPrjFace::SERIALIZED_SIZE, "faces")?,
      smoothing_groups,
      materials: Self::read_records::<T, D, _>(
        &chunks,
        Self::MATERIALS_CHUNK_ID,
        BuildPrjMaterial::SERIALIZED_SIZE,
        "materials",
      )?,
      render_shaders: Self::read_records::<T, D, _>(
        &chunks,
        Self::RENDER_SHADERS_CHUNK_ID,
        BuildPrjShader::SERIALIZED_SIZE,
        "render shaders",
      )?,
      compiler_shaders: Self::read_records::<T, D, _>(
        &chunks,
        Self::COMPILER_SHADERS_CHUNK_ID,
        BuildPrjShader::SERIALIZED_SIZE,
        "compiler shaders",
      )?,
      textures: Self::read_records::<T, D, _>(
        &chunks,
        Self::TEXTURES_CHUNK_ID,
        BuildPrjTexture::SERIALIZED_SIZE,
        "textures",
      )?,
      glows: Self::read_records::<T, D, _>(&chunks, Self::GLOWS_CHUNK_ID, BuildPrjGlow::SERIALIZED_SIZE, "glows")?,
      portals: Self::read_records::<T, D, _>(
        &chunks,
        Self::PORTALS_CHUNK_ID,
        BuildPrjPortal::SERIALIZED_SIZE,
        "portals",
      )?,
      light_controls,
      static_lights: Self::read_records::<T, D, _>(
        &chunks,
        Self::STATIC_LIGHTS_CHUNK_ID,
        LevelDynamicLight::SERIALIZED_SIZE,
        "static lights",
      )?,
      dynamic_lights: Self::read_records::<T, D, _>(
        &chunks,
        Self::DYNAMIC_LIGHTS_CHUNK_ID,
        BuildPrjDynamicLight::SERIALIZED_SIZE,
        "dynamic lights",
      )?,
      lods: Self::read_records::<T, D, _>(&chunks, Self::LODS_CHUNK_ID, BuildPrjLod::SERIALIZED_SIZE, "impostors")?,
      mu_models,
      mu_references: Self::read_records::<T, D, _>(
        &chunks,
        Self::MU_REFERENCES_CHUNK_ID,
        BuildPrjMuReference::SERIALIZED_SIZE,
        "multi-use model references",
      )?,
    })
  }

  /// Writes the project back in the order `SceneBuilder::SaveBuild` writes it in, smoothing groups after the faces.
  ///
  /// # Errors
  ///
  /// Returns an error when a multi-use model's smoothing groups disagree with the version, a record does not fit its
  /// fields, or the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if let Some(model) = self
      .mu_models
      .iter()
      .find(|model| model.smoothing_groups.is_some() != Self::has_smoothing_groups(self.version))
    {
      return Err(XrfError::new_invalid_error(format!(
        "Multi-use model '{}' {} smoothing groups, which a version {} project {}",
        model.name,
        if model.smoothing_groups.is_some() {
          "carries"
        } else {
          "lacks"
        },
        self.version,
        if Self::has_smoothing_groups(self.version) {
          "requires"
        } else {
          "has no room for"
        }
      )));
    }

    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u32::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    Self::write_records::<T, _>(writer, Self::PARAMS_CHUNK_ID, std::slice::from_ref(&self.params))?;
    Self::write_records::<T, _>(writer, Self::VERTICES_CHUNK_ID, &self.vertices)?;
    Self::write_records::<T, _>(writer, Self::FACES_CHUNK_ID, &self.faces)?;

    if let Some(groups) = &self.smoothing_groups {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      for group in groups {
        chunk.write_u32::<T>(*group)?;
      }

      chunk.flush_chunk_into::<T>(&mut writer.buffer, Self::SMOOTHING_GROUPS_CHUNK_ID)?;
    }

    Self::write_records::<T, _>(writer, Self::MATERIALS_CHUNK_ID, &self.materials)?;
    Self::write_records::<T, _>(writer, Self::RENDER_SHADERS_CHUNK_ID, &self.render_shaders)?;
    Self::write_records::<T, _>(writer, Self::COMPILER_SHADERS_CHUNK_ID, &self.compiler_shaders)?;
    Self::write_records::<T, _>(writer, Self::TEXTURES_CHUNK_ID, &self.textures)?;
    Self::write_records::<T, _>(writer, Self::GLOWS_CHUNK_ID, &self.glows)?;
    Self::write_records::<T, _>(writer, Self::PORTALS_CHUNK_ID, &self.portals)?;
    Self::write_records::<T, _>(writer, Self::LIGHT_CONTROLS_CHUNK_ID, &self.light_controls)?;
    Self::write_records::<T, _>(writer, Self::STATIC_LIGHTS_CHUNK_ID, &self.static_lights)?;
    Self::write_records::<T, _>(writer, Self::DYNAMIC_LIGHTS_CHUNK_ID, &self.dynamic_lights)?;
    Self::write_records::<T, _>(writer, Self::LODS_CHUNK_ID, &self.lods)?;

    let mut mu_models: ChunkWriter = ChunkWriter::new();

    for model in &self.mu_models {
      model.write::<T>(&mut mu_models)?;
    }

    mu_models.flush_chunk_into::<T>(&mut writer.buffer, Self::MU_MODELS_CHUNK_ID)?;

    Self::write_records::<T, _>(writer, Self::MU_REFERENCES_CHUNK_ID, &self.mu_references)?;

    Ok(())
  }

  /// Whether a project of `version` stores smoothing groups, which xrLC asserts for every multi-use model.
  pub const fn has_smoothing_groups(version: u32) -> bool {
    version >= Self::CURRENT_VERSION
  }

  /// Reads the packed run of records chunk `id` holds.
  fn read_records<T: ByteOrder, D: ChunkDataSource, R: ChunkReadWrite>(
    chunks: &[ChunkReader<D>],
    id: u32,
    record_size: u64,
    what: &str,
  ) -> XrfResult<Vec<R>> {
    ChunkReader::find_required_by_id(chunks, id)?
      .read_xr_records::<T, R>(record_size, &format!("build project {what} chunk"))
  }

  /// Reads the smoothing groups chunk, a packed run of words no record type wraps.
  fn read_smoothing_groups<T: ByteOrder, D: ChunkDataSource>(chunk: &mut ChunkReader<D>) -> XrfResult<Vec<u32>> {
    let size: u64 = chunk.read_bytes_remain();

    if !size.is_multiple_of(4) {
      return Err(XrfError::new_invalid_error(format!(
        "Build project smoothing groups chunk of {size} bytes does not divide into words"
      )));
    }

    let mut words: Vec<u32> = chunk.new_bounded_vec(size / 4, 4, "build project smoothing groups")?;

    for _ in 0..size / 4 {
      words.push(chunk.read_u32::<T>()?);
    }

    Ok(words)
  }

  /// Writes `records` as one chunk under `id`.
  fn write_records<T: ByteOrder, R: ChunkReadWrite>(writer: &mut ChunkWriter, id: u32, records: &[R]) -> XrfResult {
    let mut chunk: ChunkWriter = ChunkWriter::new();

    for record in records {
      chunk.write_xr::<T, _>(record)?;
    }

    chunk.flush_chunk_into::<T>(&mut writer.buffer, id)?;

    Ok(())
  }
}
