//! The thumbnails of the SDK's other subjects: sounds, objects and groups, and telling them from a texture's.

use image::{Rgba, RgbaImage};
use xrf_error::XrfResult;

use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::object::thm_group_file::ThmGroupFile;
use crate::object::thm_object_file::ThmObjectFile;
use crate::sound::thm_sound_file::ThmSoundFile;
use crate::tests::fixtures::{descriptor, raw_chunk, read_chunk_ids};
use crate::thm_file::ThmFile;
use crate::thm_kind::ThmKind;

/// A preview whose every texel differs from its mirror, so a flip in either direction shows.
fn preview() -> RgbaImage {
  RgbaImage::from_fn(ThmThumbnailChunk::SIZE, ThmThumbnailChunk::SIZE, |x, y| {
    Rgba([x as u8, y as u8, (x ^ y) as u8, 255 - y as u8])
  })
}

#[test]
fn a_sound_thumbnail_round_trips_in_the_sdk_order() -> XrfResult {
  let sound: ThmSoundFile = ThmSoundFile {
    quality: 0.4,
    min_distance: 2.0,
    max_distance: 80.0,
    game_type: 0x8020_0000,
    ..ThmSoundFile::new_sound()
  };
  let bytes: Vec<u8> = sound.write_to_bytes()?;

  assert_eq!(ThmSoundFile::read_from_bytes(bytes.clone())?, sound);
  assert_eq!(
    read_chunk_ids(&bytes),
    [0x0810, 0x0813, 0x1000, 0x1001, 0x1002],
    "version, type, parameters, base volume, AI distance, as ESoundThumbnail::Save writes them"
  );
  assert_eq!(ThmKind::detect(bytes)?, Some(ThmKind::Sound));

  Ok(())
}

#[test]
fn an_older_sound_thumbnail_falls_back_as_the_sdk_reads_it() -> XrfResult {
  // Version, type and the first parameter chunk alone: neither base volume nor AI distance had chunks yet.
  let bytes: Vec<u8> = [
    raw_chunk(0x0810, &ThmSoundFile::VERSION.to_le_bytes()),
    raw_chunk(0x0813, &ThmSoundFile::THUMBNAIL_TYPE.to_le_bytes()),
    raw_chunk(
      0x1000,
      &[
        0.0f32.to_le_bytes(),
        1.0f32.to_le_bytes(),
        50.0f32.to_le_bytes(),
        0u32.to_le_bytes(),
      ]
      .concat(),
    ),
  ]
  .concat();
  let sound: ThmSoundFile = ThmSoundFile::read_from_bytes(bytes.clone())?;

  assert_eq!(sound.get_base_volume(), 1.0);
  assert_eq!(sound.get_max_ai_distance(), 50.0);
  assert_eq!(
    sound.write_to_bytes()?,
    bytes,
    "chunks it lacked are not invented on the way back"
  );

  Ok(())
}

#[test]
fn an_object_thumbnail_round_trips_with_its_preview() -> XrfResult {
  let object: ThmObjectFile = ThmObjectFile {
    thumbnail: ThmThumbnailChunk::encode(&preview())?,
    face_count: 1200,
    vertex_count: 640,
    extra: Vec::new(),
    trailing: Vec::new(),
  };
  let bytes: Vec<u8> = object.write_to_bytes()?;
  let read: ThmObjectFile = ThmObjectFile::read_from_bytes(bytes.clone())?;

  assert_eq!(read, object);
  assert_eq!(read.thumbnail.decode()?, preview());
  assert_eq!(
    read_chunk_ids(&bytes),
    [0x0810, 0x8000_0811, 0x0813, 0x0816],
    "the preview compressed, as EObjectThumbnail::Save writes it"
  );
  assert_eq!(ThmKind::detect(bytes)?, Some(ThmKind::Object));

  Ok(())
}

#[test]
fn a_group_thumbnail_is_told_from_an_objects_by_its_version() -> XrfResult {
  // The SDK stamps a group's thumbnail with the object type, so only the version tells the two apart.
  let group: ThmGroupFile = ThmGroupFile {
    thumbnail: ThmThumbnailChunk::encode(&preview())?,
    objects: vec![String::from("trees\\tree_pine_01"), String::from("rocks\\rock_02")],
    extra: Vec::new(),
    trailing: Vec::new(),
  };
  let bytes: Vec<u8> = group.write_to_bytes()?;

  assert_eq!(ThmGroupFile::read_from_bytes(bytes.clone())?, group);
  assert_eq!(ThmKind::detect(bytes.clone())?, Some(ThmKind::Group));
  assert!(ThmObjectFile::read_from_bytes(bytes).is_err());

  Ok(())
}

#[test]
fn detects_a_texture_descriptor_and_refuses_reading_it_as_another_kind() -> XrfResult {
  assert_eq!(ThmKind::detect(descriptor())?, Some(ThmKind::Texture));
  assert!(ThmSoundFile::read_from_bytes(descriptor()).is_err());
  assert!(ThmObjectFile::read_from_bytes(descriptor()).is_err());
  assert_eq!(ThmKind::detect(raw_chunk(0x0810, &0x0099u16.to_le_bytes()))?, None);

  Ok(())
}

#[test]
fn refuses_a_preview_of_another_size() {
  assert!(ThmThumbnailChunk::encode(&RgbaImage::new(64, 64)).is_err());
  assert!(
    ThmThumbnailChunk {
      is_compressed: false,
      data: vec![0; 16],
    }
    .decode()
    .is_err()
  );
}

#[test]
fn a_type_change_brings_the_sdk_editors_defaults() {
  use crate::texture::thm_format::ThmFormat;
  use crate::texture::thm_mip_filter::ThmMipFilter;
  use crate::texture::thm_texture_flag::ThmTextureFlag;
  use crate::texture::thm_texture_type::ThmTextureType;

  let mut descriptor: ThmFile = ThmFile::new_texture();

  descriptor.change_texture_type(ThmTextureType::NormalMap);

  let param = descriptor.texture_param.expect("a new descriptor has parameters");

  assert_eq!(descriptor.texture_type, Some(ThmTextureType::NormalMap));
  assert_eq!(
    (param.format, param.mip_filter),
    (ThmFormat::Rgba, ThmMipFilter::Kaiser)
  );
  assert!(param.flags.has(ThmTextureFlag::GenerateMipMaps));
  assert!(!param.flags.has(ThmTextureFlag::DitherColor));

  descriptor.change_texture_type(ThmTextureType::Terrain);

  let param = descriptor.texture_param.expect("a new descriptor has parameters");

  assert_eq!(param.format, ThmFormat::Dxt1);
  assert!(param.flags.has(ThmTextureFlag::ImplicitLighted));
  assert!(!param.flags.has(ThmTextureFlag::GenerateMipMaps));

  // An image keeps whatever the descriptor had.
  descriptor.change_texture_type(ThmTextureType::Image);

  assert_eq!(descriptor.texture_param, Some(param));
}
