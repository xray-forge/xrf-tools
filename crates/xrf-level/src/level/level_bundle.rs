use std::path::Path;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::{ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::dynamic_lights::level_dynamic_light::LevelDynamicLight;
use crate::level::dynamic_lights::level_dynamic_lights_chunk::LevelDynamicLightsChunk;
use crate::level::glows::level_glow::LevelGlow;
use crate::level::glows::level_glows_chunk::LevelGlowsChunk;
use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::level::portals::level_portal::LevelPortal;
use crate::level::portals::level_portals_chunk::LevelPortalsChunk;
use crate::level::sectors::level_sector::LevelSector;
use crate::level::sectors::level_sectors_chunk::LevelSectorsChunk;
use crate::level::shaders::level_shaders_chunk::LevelShadersChunk;
use crate::level::visuals::level_visual_record::LevelVisualRecord;
use crate::level::visuals::level_visuals_chunk::LevelVisualsChunk;

/// A compiled level's `level` file held whole, as xrLC writes it (`CBuild::Run` and `RunAfterLight`,
/// `xrLC/Build.cpp`): the header, the portals, the dynamic lights, the glows, the visuals, the shader table and the
/// sectors, in that order. A chunk absent when read stays absent when written.
#[derive(Debug)]
pub struct LevelBundle {
  pub header: LevelHeaderChunk,
  pub portals: Option<Vec<LevelPortal>>,
  pub lights: Option<Vec<LevelDynamicLight>>,
  pub glows: Option<Vec<LevelGlow>>,
  pub visuals: Option<Vec<LevelVisualRecord>>,
  pub shaders: Option<LevelShadersChunk>,
  pub sectors: Option<Vec<LevelSector>>,
}

impl LevelBundle {
  /// Reads a `level` file whole from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or a chunk of it is not one this reads whole.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_bytes::<T>(std::fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a `level` file whole from bytes in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the header is absent, or a chunk is not one this reads whole.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    let chunks = ChunkReader::from_vec(bytes)?.read_children()?;
    let find = |id: u32| ChunkReader::find_optional_by_id(&chunks, id);

    Ok(Self {
      header: ChunkReader::find_required_by_id(&chunks, LevelHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      portals: find(LevelPortalsChunk::CHUNK_ID)
        .map(|mut chunk| LevelPortalsChunk::read_from_chunk::<T, _>(&mut chunk).map(|chunk| chunk.portals))
        .transpose()?,
      lights: find(LevelDynamicLightsChunk::CHUNK_ID)
        .map(|mut chunk| LevelDynamicLightsChunk::read_from_chunk::<T, _>(&mut chunk).map(|chunk| chunk.lights))
        .transpose()?,
      glows: find(LevelGlowsChunk::CHUNK_ID)
        .map(|mut chunk| LevelGlowsChunk::read_from_chunk::<T, _>(&mut chunk).map(|chunk| chunk.glows))
        .transpose()?,
      visuals: find(LevelVisualsChunk::CHUNK_ID)
        .map(|mut chunk| {
          LevelVisualsChunk::read_from_chunk::<T, _>(&mut chunk)?
            .visuals
            .into_iter()
            .map(LevelVisualRecord::of)
            .collect::<XrfResult<Vec<_>>>()
        })
        .transpose()?,
      shaders: find(LevelShadersChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>())
        .transpose()?,
      sectors: find(LevelSectorsChunk::CHUNK_ID)
        .map(|mut chunk| LevelSectorsChunk::read_from_chunk::<T, _>(&mut chunk).map(|chunk| chunk.sectors))
        .transpose()?,
    })
  }

  /// Writes the file in xrLC's chunk order, each visual and each sector a chunk numbered by its index.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    Self::put::<T>(writer, LevelHeaderChunk::CHUNK_ID, |chunk| {
      chunk.write_xr::<T, _>(&self.header)
    })?;

    if let Some(portals) = &self.portals {
      Self::put::<T>(writer, LevelPortalsChunk::CHUNK_ID, |chunk| {
        portals.iter().try_for_each(|portal| chunk.write_xr::<T, _>(portal))
      })?;
    }

    if let Some(lights) = &self.lights {
      Self::put::<T>(writer, LevelDynamicLightsChunk::CHUNK_ID, |chunk| {
        lights.iter().try_for_each(|light| chunk.write_xr::<T, _>(light))
      })?;
    }

    if let Some(glows) = &self.glows {
      Self::put::<T>(writer, LevelGlowsChunk::CHUNK_ID, |chunk| {
        glows.iter().try_for_each(|glow| chunk.write_xr::<T, _>(glow))
      })?;
    }

    if let Some(visuals) = &self.visuals {
      Self::put::<T>(writer, LevelVisualsChunk::CHUNK_ID, |chunk| {
        visuals
          .iter()
          .enumerate()
          .try_for_each(|(index, visual)| Self::put::<T>(chunk, index as u32, |inner| visual.write::<T>(inner)))
      })?;
    }

    if let Some(shaders) = &self.shaders {
      Self::put::<T>(writer, LevelShadersChunk::CHUNK_ID, |chunk| {
        chunk.write_xr::<T, _>(shaders)
      })?;
    }

    if let Some(sectors) = &self.sectors {
      Self::put::<T>(writer, LevelSectorsChunk::CHUNK_ID, |chunk| {
        sectors.iter().enumerate().try_for_each(|(index, sector)| {
          Self::put::<T>(chunk, index as u32, |inner| {
            Self::put::<T>(inner, LevelSector::ROOT_CHUNK_ID, |root| {
              Ok(root.write_u32::<T>(sector.root)?)
            })?;
            Self::put::<T>(inner, LevelSector::PORTALS_CHUNK_ID, |portals| {
              sector
                .portals
                .iter()
                .try_for_each(|portal| Ok(portals.write_u16::<T>(*portal)?))
            })
          })
        })
      })?;
    }

    Ok(())
  }

  /// Writes what `fill` puts in a chunk as chunk `id` of `writer`.
  fn put<T: ByteOrder>(
    writer: &mut ChunkWriter,
    id: u32,
    fill: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    let mut chunk: ChunkWriter = ChunkWriter::new();

    fill(&mut chunk)?;
    chunk.flush_chunk_into::<T>(&mut writer.buffer, id)?;

    Ok(())
  }
}
