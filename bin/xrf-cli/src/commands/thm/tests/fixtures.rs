//! A thumbnail of each kind the `thm` commands meet, written through `xrf-thm` in the order the SDK writes them.

use xrf_dds::{Rgba, RgbaImage};
use xrf_error::XrfResult;
use xrf_thm::{ThmBumpChunk, ThmBumpMode, ThmFile, ThmGroupFile, ThmObjectFile, ThmSoundFile, ThmThumbnailChunk};

/// A compressed preview, as the SDK draws one for an object or a group.
pub(crate) fn preview() -> XrfResult<ThmThumbnailChunk> {
  ThmThumbnailChunk::encode(&RgbaImage::from_fn(
    ThmThumbnailChunk::SIZE,
    ThmThumbnailChunk::SIZE,
    |x, y| Rgba([x as u8, y as u8, 90, 255]),
  ))
}

/// A texture descriptor declaring a used bump, as most corpus descriptors do.
pub(crate) fn texture_descriptor() -> ThmFile {
  ThmFile {
    bump: Some(ThmBumpChunk {
      mode: ThmBumpMode::Use,
      name: String::from("ston\\ston_beton05_bump"),
      ..ThmBumpChunk::default()
    }),
    ..ThmFile::new_texture()
  }
}

/// An object's thumbnail, its preview and size.
pub(crate) fn object_thumbnail() -> XrfResult<ThmObjectFile> {
  Ok(ThmObjectFile {
    thumbnail: preview()?,
    face_count: 1200,
    vertex_count: 640,
    extra: Vec::new(),
    trailing: Vec::new(),
  })
}

/// A group's thumbnail, which the SDK stamps with the object type, its preview and the objects it gathers.
pub(crate) fn group_thumbnail() -> XrfResult<ThmGroupFile> {
  Ok(ThmGroupFile {
    thumbnail: preview()?,
    objects: vec![String::from("trees\\tree_pine_01"), String::from("rocks\\rock_02")],
    extra: Vec::new(),
    trailing: Vec::new(),
  })
}

/// A sound's thumbnail, quieter and nearer than the constructor's.
pub(crate) fn sound_thumbnail() -> ThmSoundFile {
  ThmSoundFile {
    quality: 0.4,
    min_distance: 2.0,
    max_distance: 80.0,
    ..ThmSoundFile::new_sound()
  }
}

/// One chunk laid out the way the format does: id, payload length, then the payload.
pub(crate) fn raw_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  [&id.to_le_bytes()[..], &(payload.len() as u32).to_le_bytes(), payload].concat()
}
