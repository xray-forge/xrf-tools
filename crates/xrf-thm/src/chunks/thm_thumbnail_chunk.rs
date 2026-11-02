use std::io::Write;

use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use xrf_chunk::{CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_MASK, ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_lzhuf::{LzhufDecoder, LzhufEncoder};

/// The preview picture a thumbnail carries, `THM_CHUNK_DATA` in the engine (`ETextureParams.h`).
///
/// The SDK writes it as `THUMB_WIDTH` by `THUMB_HEIGHT` `D3DCOLOR`s, the bottom row first as `DrawThumbnail` reads
/// them, coded with `_compressLZ` (`EObjectThumbnail::Save`). A texture descriptor carries one only from an SDK older
/// than the one that stopped writing it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmThumbnailChunk {
  /// Whether the payload is the engine's compressed stream, which is the only form seen in the wild.
  pub is_compressed: bool,
  /// The payload exactly as the file stores it.
  pub data: Vec<u8>,
}

impl ThmThumbnailChunk {
  pub const CHUNK_ID: u32 = 0x0811;

  /// `THUMB_WIDTH` and `THUMB_HEIGHT`, the preview's size in texels.
  pub const SIZE: u32 = 128;

  /// The preview as a picture, top row first.
  ///
  /// # Errors
  ///
  /// Returns an error when a compressed payload does not decode, or the picture is not [`Self::SIZE`] square.
  pub fn decode(&self) -> XrfResult<RgbaImage> {
    let bytes: Vec<u8> = if self.is_compressed {
      LzhufDecoder::decompress(&self.data)?
    } else {
      self.data.clone()
    };
    let expected: usize = Self::SIZE as usize * Self::SIZE as usize * 4;

    if bytes.len() != expected {
      return Err(XrfError::new_parsing_error(format!(
        "Thumbnail preview holds {} bytes, not the {expected} of a {}x{} picture",
        bytes.len(),
        Self::SIZE,
        Self::SIZE
      )));
    }

    Ok(RgbaImage::from_fn(Self::SIZE, Self::SIZE, |x, y| {
      let offset: usize = ((Self::SIZE - 1 - y) * Self::SIZE + x) as usize * 4;
      let [blue, green, red, alpha]: [u8; 4] = bytes[offset..offset + 4].try_into().expect("four bytes a texel");

      Rgba([red, green, blue, alpha])
    }))
  }

  /// A compressed preview of `picture`, as the SDK writes an object's or a group's.
  ///
  /// # Errors
  ///
  /// Returns an error when the picture is not [`Self::SIZE`] square.
  pub fn encode(picture: &RgbaImage) -> XrfResult<Self> {
    if picture.dimensions() != (Self::SIZE, Self::SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Thumbnail preview is {}x{}, not the {}x{} the SDK draws",
        picture.width(),
        picture.height(),
        Self::SIZE,
        Self::SIZE
      )));
    }

    let bytes: Vec<u8> = (0..Self::SIZE)
      .rev()
      .flat_map(|y| (0..Self::SIZE).map(move |x| (x, y)))
      .flat_map(|(x, y)| {
        let [red, green, blue, alpha] = picture.get_pixel(x, y).0;

        [blue, green, red, alpha]
      })
      .collect();

    Ok(Self {
      is_compressed: true,
      data: LzhufEncoder::compress(&bytes)?,
    })
  }

  /// Whether a chunk id names this chunk, in either of the two forms a writer can have marked it with.
  pub const fn matches(id: u32) -> bool {
    id & CHUNK_ID_MASK == Self::CHUNK_ID
  }

  /// Reads the payload of a chunk found under `id`, which is what says whether it is compressed.
  pub fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>, id: u32) -> XrfResult<Self> {
    Ok(Self {
      is_compressed: id & CHUNK_ID_COMPRESSED_MASK != 0,
      data: reader.read_remaining()?,
    })
  }

  /// The id this chunk is written back under.
  pub const fn to_chunk_id(&self) -> u32 {
    if self.is_compressed {
      Self::CHUNK_ID | CHUNK_ID_COMPRESSED_MASK
    } else {
      Self::CHUNK_ID
    }
  }

  /// Writes the payload, which is copied rather than re-encoded.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.data)?;

    Ok(())
  }
}
