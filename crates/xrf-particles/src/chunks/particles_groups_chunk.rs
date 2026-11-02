use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{FileImportExport, Ltx, LtxImportExport, META_TYPE_FIELD};
use xrf_utils::{format_path, to_format_size};

use crate::data::group::particle_group::ParticleGroup;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesGroupsChunk {
  pub groups: Vec<ParticleGroup>,
}

impl ParticlesGroupsChunk {
  pub const CHUNK_ID: u32 = 4;

  /// The group by its exact name, as `CPSLibrary::FindPGD` looks one up.
  pub fn get(&self, name: &str) -> Option<&ParticleGroup> {
    self.groups.iter().find(|group| group.name == name)
  }

  pub fn get_mut(&mut self, name: &str) -> Option<&mut ParticleGroup> {
    self.groups.iter_mut().find(|group| group.name == name)
  }

  /// Adds a group where its name sorts, the order the engine looks groups up in and a library is read in.
  ///
  /// # Errors
  ///
  /// Returns an error when a group already has its name.
  pub fn insert(&mut self, group: ParticleGroup) -> XrfResult {
    match self.groups.binary_search_by(|it| it.name.cmp(&group.name)) {
      Ok(_) => Err(XrfError::new_invalid_error(format!(
        "Particle group '{}' is already defined",
        group.name
      ))),
      Err(position) => {
        self.groups.insert(position, group);

        Ok(())
      }
    }
  }

  /// Takes the group by its exact name out, as `CPSLibrary::Remove` does.
  pub fn remove(&mut self, name: &str) -> Option<ParticleGroup> {
    let position: usize = self.groups.iter().position(|group| group.name == name)?;

    Some(self.groups.remove(position))
  }
}

impl ChunkReadWrite for ParticlesGroupsChunk {
  /// Read effects chunk by position descriptor.
  /// Parses binary data into version chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut groups: Vec<ParticleGroup> = Vec::new();

    log::info!(
      "Parsed groups chunk, {} bytes, {} chunks",
      reader.read_bytes_len(),
      chunks.len()
    );

    for mut chunk_reader in chunks {
      groups.push(chunk_reader.read_xr::<T, _>()?);
    }

    groups.sort_by(|first, second| first.name.cmp(&second.name));

    reader.assert_read("Expect groups chunk to be ended")?;

    Ok(Self { groups })
  }

  /// Write particle groups data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, group) in self.groups.iter().enumerate() {
      let mut group_writer: ChunkWriter = ChunkWriter::new();

      group_writer.write_xr::<T, _>(group)?;
      writer
        .write_all(&group_writer.flush_chunk_into_buffer::<T>(to_format_size(index, "particle group chunk id")?)?)?;
    }

    log::info!("Written groups chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for ParticlesGroupsChunk {
  /// Import particles groups data from provided path.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    log::info!("Importing particles groups: {}", format_path(path.as_ref()));

    let ltx: Ltx = Ltx::read_from_path(path.as_ref().join("groups.ltx"))?;
    let mut groups: Vec<ParticleGroup> = Vec::new();

    for (section_name, section) in &ltx {
      if let Some(meta_field) = section.get(META_TYPE_FIELD)
        && meta_field == ParticleGroup::META_TYPE
      {
        groups.push(ParticleGroup::import(section_name, &ltx)?);
      }
    }

    groups.sort_by(|first, second| first.name.cmp(&second.name));

    Ok(Self { groups })
  }

  /// Export particles groups data into provided path.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut particles_effects_ltx: Ltx = Ltx::new();

    for group in &self.groups {
      group.export(&group.name, &mut particles_effects_ltx)?;
    }

    particles_effects_ltx.write_to_path(path.as_ref().join("groups.ltx"))?;

    log::info!("Exported groups chunk");

    Ok(())
  }
}
