use byteorder::{ByteOrder, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, META_TYPE_FIELD, Section};
use xrf_utils::assert_equal;

use crate::data::group::particle_group_effect::ParticleGroupEffect;
use crate::data::group::particle_group_effect_old::ParticleGroupEffectOld;
use crate::data::particle_description::ParticleDescription;
use crate::data::particle_record_chunks::ParticleRecordChunks;

/// A sequence of effects, the engine's `CPGDef` (`Layers/xrRender/ParticleGroup.cpp`).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleGroup {
  pub version: u16,
  pub name: String,
  pub flags: u32,
  /// Seconds the group plays for as stored; the engine plays it to its latest effect's `time_1` where this is not
  /// above zero ([`Self::get_time_limit`]).
  pub time_limit: f32,
  pub effects: Vec<ParticleGroupEffect>,
  pub description: Option<ParticleDescription>,
  /// The effects in Shadow of Chernobyl's earlier layout, which the engine of that game reads where `effects` is absent.
  pub effects_old: Option<Vec<ParticleGroupEffectOld>>,
}

impl ParticleGroup {
  pub const META_TYPE: &'static str = "particle_group";

  /// `PGD_VERSION`, the one version `CPGDef::Load` reads; any other fails the whole library's load.
  pub const VERSION: u16 = 3;

  pub const VERSION_CHUNK_ID: u32 = 1;
  pub const NAME_CHUNK_ID: u32 = 2;
  pub const FLAGS_CHUNK_ID: u32 = 3;
  pub const EFFECTS_CHUNK_ID: u32 = 4;
  pub const TIME_LIMIT_CHUNK_ID: u32 = 5;
  pub const DESCRIPTION_CHUNK_ID: u32 = 6;
  pub const EFFECTS2_CHUNK_ID: u32 = 7;

  fn get_description_section(section_name: &str) -> String {
    format!("{section_name}.description")
  }

  /// Seconds the group plays for, as `CPGDef::Load` settles them: the stored limit where it is above zero, otherwise
  /// the latest `time_1` of its effects; zero or less loops.
  pub fn get_time_limit(&self) -> f32 {
    if self.time_limit > 0.0 {
      self.time_limit
    } else {
      self
        .effects
        .iter()
        .fold(self.time_limit, |limit, effect| limit.max(effect.time_1))
    }
  }
}

impl ChunkReadWrite for ParticleGroup {
  /// Reads a group as `CPGDef::Load` does: its effects and time limit may be absent, which reads as none and zero.
  ///
  /// # Errors
  ///
  /// Returns a parsing error naming the chunk when a required one is missing or any is malformed, and when the version
  /// is not [`Self::VERSION`].
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let version: u16 = ParticleRecordChunks::read_required(&chunks, Self::VERSION_CHUNK_ID, "version", |it| {
      it.read_u16_payload::<T>()
    })?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Failed to read particle group of version {version}, only version {} is read by the engine",
        Self::VERSION
      )));
    }

    let particle_group: Self = Self {
      version,
      name: ParticleRecordChunks::read_required(&chunks, Self::NAME_CHUNK_ID, "name", |it| {
        it.read_w1251_string_payload()
      })?,
      flags: ParticleRecordChunks::read_required(&chunks, Self::FLAGS_CHUNK_ID, "flags", |it| {
        it.read_u32_payload::<T>()
      })?,
      effects: ParticleRecordChunks::read_optional(&chunks, Self::EFFECTS_CHUNK_ID, "effects", |it| {
        it.read_xr_list::<T, _>()
      })?
      .unwrap_or_default(),
      time_limit: ParticleRecordChunks::read_optional(&chunks, Self::TIME_LIMIT_CHUNK_ID, "time_limit", |it| {
        it.read_f32_payload::<T>()
      })?
      .unwrap_or(0.0),
      description: ParticleRecordChunks::read_optional(&chunks, Self::DESCRIPTION_CHUNK_ID, "description", |it| {
        ParticleDescription::read::<T, _>(it)
      })?,
      effects_old: ParticleRecordChunks::read_optional(&chunks, Self::EFFECTS2_CHUNK_ID, "old effects", |it| {
        it.read_xr_list::<T, _>()
      })?,
    };

    reader.assert_read("Expect groups chunk to be ended")?;

    Ok(particle_group)
  }

  /// Write particle group data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version_chunk_writer: ChunkWriter = ChunkWriter::new();
    version_chunk_writer.write_u16::<T>(self.version)?;
    version_chunk_writer.flush_chunk_into::<T>(writer, Self::VERSION_CHUNK_ID)?;

    let mut name_chunk_writer: ChunkWriter = ChunkWriter::new();
    name_chunk_writer.write_w1251_string(&self.name)?;
    name_chunk_writer.flush_chunk_into::<T>(writer, Self::NAME_CHUNK_ID)?;

    let mut flags_chunk_writer: ChunkWriter = ChunkWriter::new();
    flags_chunk_writer.write_u32::<T>(self.flags)?;
    flags_chunk_writer.flush_chunk_into::<T>(writer, Self::FLAGS_CHUNK_ID)?;

    let mut effects_chunk_writer: ChunkWriter = ChunkWriter::new();
    effects_chunk_writer.write_xr_list::<T, _>(&self.effects)?;
    effects_chunk_writer.flush_chunk_into::<T>(writer, Self::EFFECTS_CHUNK_ID)?;

    let mut time_limit_chunk_writer: ChunkWriter = ChunkWriter::new();
    time_limit_chunk_writer.write_f32::<T>(self.time_limit)?;
    time_limit_chunk_writer.flush_chunk_into::<T>(writer, Self::TIME_LIMIT_CHUNK_ID)?;

    if let Some(description) = &self.description {
      let mut description_chunk_writer: ChunkWriter = ChunkWriter::new();
      description.write::<T>(&mut description_chunk_writer)?;
      description_chunk_writer.flush_chunk_into::<T>(writer, Self::DESCRIPTION_CHUNK_ID)?;
    }

    if let Some(effects_old) = &self.effects_old
      && !effects_old.is_empty()
    {
      let mut effects_old_chunk_writer: ChunkWriter = ChunkWriter::new();
      effects_old_chunk_writer.write_xr_list::<T, _>(effects_old)?;
      effects_old_chunk_writer.flush_chunk_into::<T>(writer, Self::EFFECTS2_CHUNK_ID)?;
    }

    Ok(())
  }
}

impl LtxImportExport for ParticleGroup {
  /// Import particles group data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle group section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = section.read_field(META_TYPE_FIELD)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle group importing",
    )?;

    let effects_old: Vec<ParticleGroupEffectOld> = ParticleGroupEffectOld::import_list(section_name, ltx)?;

    Ok(Self {
      version: section.read_field("version")?,
      name: section.read_field("name")?,
      flags: section.read_field("flags")?,
      time_limit: section.read_field("time_limit")?,
      effects: ParticleGroupEffect::import_list(section_name, ltx)?,
      description: ParticleDescription::import_optional(&Self::get_description_section(section_name), ltx)?,
      effects_old: if effects_old.is_empty() {
        None
      } else {
        Some(effects_old)
      },
    })
  }

  /// Export particles group data into provided path.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set("version", self.version.to_string())
      .set("name", &self.name)
      .set("flags", self.flags.to_string())
      .set("time_limit", self.time_limit.to_string());

    ParticleGroupEffect::export_list(&self.effects, section_name, ltx)?;

    if let Some(description) = &self.description {
      description.export(&Self::get_description_section(section_name), ltx)?;
    }

    if let Some(effects_old) = &self.effects_old {
      ParticleGroupEffectOld::export_list(effects_old, section_name, ltx)?;
    }

    Ok(())
  }
}
