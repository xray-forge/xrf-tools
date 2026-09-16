use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// The gametype word that admits every mode, which is what a single-player placement carries.
pub const PS_STATIC_ALL_GAME_TYPES: u16 = u16::MAX;

/// One particle effect the level plants, from `CLevel::Load`'s static particle pass.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PsStaticPlacement {
  /// Modes this placement plays in. Version 0 carries none and plays in all of them.
  pub game_types: Option<u16>,
  /// The effect it plays, which names a definition inside `particles.xr` rather than a file.
  pub effect: String,
  /// `Fmatrix`, row major, the fourth row being the translation the engine nudges up by a centimetre on load.
  pub transform: [f32; 16],
}

impl PsStaticPlacement {
  /// Bytes the transform occupies.
  pub const TRANSFORM_SIZE: u64 = 16 * 4;

  /// Where the placement sits, which is the transform's translation row.
  pub const fn get_position(&self) -> Vector3d<f32> {
    Vector3d {
      x: self.transform[12],
      y: self.transform[13],
      z: self.transform[14],
    }
  }

  /// Whether the placement plays in every mode, which all but a handful of multiplayer ones do.
  pub fn is_every_game_type(&self) -> bool {
    self.game_types.is_none_or(|types| types == PS_STATIC_ALL_GAME_TYPES)
  }

  /// Reads one placement.
  ///
  /// Takes the version rather than reading one, because the file holds it once ahead of every record and it decides
  /// whether a gametype word is there at all.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the record.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, version: u32) -> XrfResult<Self> {
    let game_types: Option<u16> = if version > 0 {
      Some(reader.read_u16::<T>()?)
    } else {
      None
    };

    let effect: String = reader.read_w1251_string()?;
    let mut transform: [f32; 16] = [0.0; 16];

    for value in &mut transform {
      *value = reader.read_f32::<T>()?;
    }

    Ok(Self {
      game_types,
      effect,
      transform,
    })
  }

  /// Writes one placement back.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if let Some(game_types) = self.game_types {
      writer.write_u16::<T>(game_types)?;
    }

    writer.write_w1251_string(&self.effect)?;

    for value in self.transform {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
