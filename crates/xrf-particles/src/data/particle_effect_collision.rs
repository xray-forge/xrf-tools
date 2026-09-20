use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, META_TYPE_FIELD, Section, read_ltx_field};
use xrf_utils::assert_equal;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffectCollision {
  pub collide_one_minus_friction: f32,
  pub collide_resilience: f32,
  pub collide_sqr_cutoff: f32,
}

impl ParticleEffectCollision {
  pub const META_TYPE: &'static str = "particle_effect_collision";
}

impl ChunkReadWrite for ParticleEffectCollision {
  /// Read particle effect collision data from chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let particle_collision: Self = Self {
      collide_one_minus_friction: reader.read_f32::<T>()?,
      collide_resilience: reader.read_f32::<T>()?,
      collide_sqr_cutoff: reader.read_f32::<T>()?,
    };

    reader.assert_read("Expect particle collision chunk to be ended")?;

    Ok(particle_collision)
  }

  /// Write particle effect collision data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.collide_one_minus_friction)?;
    writer.write_f32::<T>(self.collide_resilience)?;
    writer.write_f32::<T>(self.collide_sqr_cutoff)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleEffectCollision {
  /// Import particle effect collision data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle effect description section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = read_ltx_field(META_TYPE_FIELD, section)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle effect collision importing",
    )?;

    Ok(Self {
      collide_one_minus_friction: read_ltx_field("collide_one_minus_friction", section)?,
      collide_resilience: read_ltx_field("collide_resilience", section)?,
      collide_sqr_cutoff: read_ltx_field("collide_sqr_cutoff", section)?,
    })
  }

  /// Export particle effect collision data into provided path.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set(
        "collide_one_minus_friction",
        self.collide_one_minus_friction.to_string(),
      )
      .set("collide_resilience", self.collide_resilience.to_string())
      .set("collide_sqr_cutoff", self.collide_sqr_cutoff.to_string());

    Ok(())
  }
}
