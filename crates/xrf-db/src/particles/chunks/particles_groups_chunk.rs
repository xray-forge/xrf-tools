use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{format_path, open_export_file, to_format_size};

use crate::constants::META_TYPE_FIELD;
use crate::data::particles::particle_group::ParticleGroup;
use crate::export::{FileImportExport, LtxImportExport};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesGroupsChunk {
  pub groups: Vec<ParticleGroup>,
}

impl ParticlesGroupsChunk {
  pub const CHUNK_ID: u32 = 4;
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

    particles_effects_ltx.write_to(&mut open_export_file(path.as_ref().join("groups.ltx"))?)?;

    log::info!("Exported groups chunk");

    Ok(())
  }
}
