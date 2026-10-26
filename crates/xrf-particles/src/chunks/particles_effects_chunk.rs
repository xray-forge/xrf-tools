use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{FileImportExport, Ltx, LtxImportExport, META_TYPE_FIELD};
use xrf_utils::{assert, format_path, to_format_size};

use crate::data::effect::particle_effect::ParticleEffect;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesEffectsChunk {
  pub effects: Vec<ParticleEffect>,
}

impl ParticlesEffectsChunk {
  pub const CHUNK_ID: u32 = 3;

  /// The effect by its exact name, as `CPSLibrary::FindPED` looks one up.
  pub fn get(&self, name: &str) -> Option<&ParticleEffect> {
    self.effects.iter().find(|effect| effect.name == name)
  }

  pub fn get_mut(&mut self, name: &str) -> Option<&mut ParticleEffect> {
    self.effects.iter_mut().find(|effect| effect.name == name)
  }

  /// Adds an effect where its name sorts, the order the engine looks effects up in and a library is read in.
  ///
  /// # Errors
  ///
  /// Returns an error when an effect already has its name.
  pub fn insert(&mut self, effect: ParticleEffect) -> XrfResult {
    match self.effects.binary_search_by(|it| it.name.cmp(&effect.name)) {
      Ok(_) => Err(XrfError::new_invalid_error(format!(
        "Particle effect '{}' is already defined",
        effect.name
      ))),
      Err(position) => {
        self.effects.insert(position, effect);

        Ok(())
      }
    }
  }

  /// Takes the effect by its exact name out, as `CPSLibrary::Remove` does.
  pub fn remove(&mut self, name: &str) -> Option<ParticleEffect> {
    let position: usize = self.effects.iter().position(|effect| effect.name == name)?;

    Some(self.effects.remove(position))
  }
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
    log::info!("Importing particles effects: {}", format_path(path.as_ref()));

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

    particles_effects_ltx.write_to_path(path.as_ref().join("effects.ltx"))?;

    log::info!("Exported effects chunk");

    Ok(())
  }
}
