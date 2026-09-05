//! Descriptor files laid down as raw bytes, in the shape the corpus holds.

use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::thm::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::thm::chunks::thm_material_chunk::ThmMaterialChunk;
use crate::thm::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use xrf_chunk::CHUNK_ID_COMPRESSED_MASK;

use crate::thm::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::thm::thm_file::ThmFile;

/// The bump the sample descriptor declares.
pub(crate) const BUMP_NAME: &str = "ston\\ston_beton05_bump";

/// The chunk ids of a descriptor as `ETextureThumbnail::Save` lays them out, without the optional thumbnail.
pub(crate) const WHOLE_FILE_ORDER: [u32; 9] = [
  ThmFile::VERSION_CHUNK_ID,
  ThmFile::THUMBNAIL_TYPE_CHUNK_ID,
  ThmTextureParamChunk::CHUNK_ID,
  ThmFile::TEXTURE_TYPE_CHUNK_ID,
  ThmDetailChunk::CHUNK_ID,
  ThmMaterialChunk::CHUNK_ID,
  ThmBumpChunk::CHUNK_ID,
  ThmFile::EXT_NORMAL_MAP_CHUNK_ID,
  ThmFile::FADE_DELAY_CHUNK_ID,
];

/// A descriptor of the shape every corpus file has: nine chunks, a used bump, no detail, no thumbnail.
pub(crate) fn descriptor() -> Vec<u8> {
  chunks()
    .into_iter()
    .flat_map(|(id, payload)| raw_chunk(id, &payload))
    .collect()
}

/// The same descriptor with a compressed thumbnail after the version chunk, as the eleven Anomaly files carry it.
pub(crate) fn descriptor_with_thumbnail(payload: &[u8]) -> Vec<u8> {
  let mut file: Vec<u8> = Vec::new();

  for (id, chunk) in chunks() {
    file.extend_from_slice(&raw_chunk(id, &chunk));

    if id == ThmFile::VERSION_CHUNK_ID {
      file.extend_from_slice(&raw_chunk(
        ThmThumbnailChunk::CHUNK_ID | CHUNK_ID_COMPRESSED_MASK,
        payload,
      ));
    }
  }

  file
}

/// The sample descriptor's chunks, in the order they sit in the file.
fn chunks() -> Vec<(u32, Vec<u8>)> {
  vec![
    (ThmFile::VERSION_CHUNK_ID, ThmFile::VERSION.to_le_bytes().to_vec()),
    (
      ThmFile::THUMBNAIL_TYPE_CHUNK_ID,
      ThmFile::THUMBNAIL_TYPE_TEXTURE.to_le_bytes().to_vec(),
    ),
    (
      ThmTextureParamChunk::CHUNK_ID,
      // fmt DXT5, flags mip maps + dither colour + has alpha, no borders or fades, box filter, 512 square.
      pack_u32(&[3, 0x0200_0101, 0, 0, 0, 0, 512, 512]),
    ),
    (ThmFile::TEXTURE_TYPE_CHUNK_ID, pack_u32(&[0])),
    (
      ThmDetailChunk::CHUNK_ID,
      [stringz(""), 1.0_f32.to_le_bytes().to_vec()].concat(),
    ),
    (
      ThmMaterialChunk::CHUNK_ID,
      [pack_u32(&[1]), 0.0_f32.to_le_bytes().to_vec()].concat(),
    ),
    (
      ThmBumpChunk::CHUNK_ID,
      [0.05_f32.to_le_bytes().to_vec(), pack_u32(&[2]), stringz(BUMP_NAME)].concat(),
    ),
    (ThmFile::EXT_NORMAL_MAP_CHUNK_ID, stringz("")),
    (ThmFile::FADE_DELAY_CHUNK_ID, vec![0]),
  ]
}

/// Lay out one chunk the way the format does: id, payload length, then the payload.
pub(crate) fn raw_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// The chunk ids of a descriptor's bytes, in the order they appear.
pub(crate) fn read_chunk_ids(bytes: &[u8]) -> Vec<u32> {
  let mut ids: Vec<u32> = Vec::new();
  let mut position: usize = 0;

  while position + 8 <= bytes.len() {
    ids.push(u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap()));

    let size: usize = u32::from_le_bytes(bytes[position + 4..position + 8].try_into().unwrap()) as usize;

    position += 8 + size;
  }

  ids
}

fn pack_u32(values: &[u32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

fn stringz(value: &str) -> Vec<u8> {
  [value.as_bytes(), &[0]].concat()
}
