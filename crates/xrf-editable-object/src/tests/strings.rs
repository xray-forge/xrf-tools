use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::EditableObjectFile;
use crate::tests::fixtures;

/// An object whose LOD reference is `value`'s bytes, terminated.
fn object_with_lods(value: &[u8]) -> Vec<u8> {
  let mut lods: Vec<u8> = value.to_vec();

  lods.push(0);

  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[0]),
  ));
  body.extend(fixtures::chunk(EditableObjectFile::LODS_CHUNK_ID, &lods));

  fixtures::file(&body)
}

#[test]
fn a_cyrillic_name_comes_back_as_its_bytes() -> XrfResult {
  // «Новая папка», the folder name one mod-shipped object sits under, in Windows-1251.
  let name: [u8; 11] = [0xcd, 0xee, 0xe2, 0xe0, 0xff, 0x20, 0xef, 0xe0, 0xef, 0xea, 0xe0];
  let original: Vec<u8> = object_with_lods(&name);
  let object: EditableObjectFile = EditableObjectFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  assert_eq!(object.lods.as_deref(), Some("Новая папка"));
  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn a_name_opening_on_the_utf8_byte_order_mark_loses_it_on_read() -> XrfResult {
  // todo: Read chunk strings without byte order mark sniffing, so a Windows-1251 name opening on «п»ї» (EF BB BF)
  // keeps those bytes; `ChunkReader::read_w1251_string` decodes it as UTF-8 and drops them, and this pins that.
  let original: Vec<u8> = object_with_lods(&[0xef, 0xbb, 0xbf, b'a']);
  let object: EditableObjectFile = EditableObjectFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  assert_eq!(object.lods.as_deref(), Some("a"));
  assert_ne!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}
