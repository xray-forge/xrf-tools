use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{assert, open_export_file, to_format_size};

use crate::constants::META_TYPE_FIELD;
use crate::data::particles::particle_effect::ParticleEffect;
use crate::export::{FileImportExport, LtxImportExport};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesEffectsChunk {
  pub effects: Vec<ParticleEffect>,
}

impl ParticlesEffectsChunk {
  pub const CHUNK_ID: u32 = 3;
}

impl ChunkReadWrite for ParticlesEffectsChunk {
  /// Read effects chunk by position descriptor.
  /// Parses binary data into version chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut effects: Vec<ParticleEffect> = Vec::new();

    log::info!(
      "Parsing effects chunk, {} bytes, {} chunks",
      reader.read_bytes_len(),
      chunks.len()
    );

    for mut chunk in chunks {
      effects.push(ParticleEffect::read::<T, _>(&mut chunk)?);
    }

    effects.sort_by(|first, second| first.name.cmp(&second.name));

    assert(reader.is_ended(), "Expect effects chunk to be ended")?;

    Ok(Self { effects })
  }

  /// Write particle effects data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, effect) in self.effects.iter().enumerate() {
      let mut effect_writer: ChunkWriter = ChunkWriter::new();

      effect.write::<T>(&mut effect_writer)?;

      writer.write_all(
        effect_writer
          .flush_chunk_into_buffer::<T>(to_format_size(index, "particle effect chunk id")?)?
          .as_slice(),
      )?;
    }

    log::info!(
      "Written effects chunk, {} bytes, {} chunks",
      writer.bytes_written(),
      self.effects.len()
    );

    Ok(())
  }
}

impl FileImportExport for ParticlesEffectsChunk {
  /// Import particles effects data from provided path.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    log::info!("Importing particles effects: {}", path.as_ref().display());

    let ltx: Ltx = Ltx::read_from_path(path.as_ref().join("effects.ltx"))?;
    let mut effects: Vec<ParticleEffect> = Vec::new();

    for (section_name, section) in &ltx {
      if let Some(meta_field) = section.get(META_TYPE_FIELD)
        && meta_field == ParticleEffect::META_TYPE
      {
        effects.push(ParticleEffect::import(section_name, &ltx)?);
      }
    }

    effects.sort_by(|first, second| first.name.cmp(&second.name));

    Ok(Self { effects })
  }

  /// Export particles effects data into provided path.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut particles_effects_ltx: Ltx = Ltx::new();

    for effect in &self.effects {
      effect.export(&effect.name, &mut particles_effects_ltx)?;
    }

    particles_effects_ltx.write_to(&mut open_export_file(path.as_ref().join("effects.ltx"))?)?;

    log::info!("Exported effects chunk");

    Ok(())
  }
}
