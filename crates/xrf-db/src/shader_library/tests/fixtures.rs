//! Blenders and libraries laid down as raw bytes, in the shape the corpus holds.
//!
//! Hand laid rather than written through [`ShaderBlender`]'s own writer, because a reader cannot be judged against
//! fixtures its writer produced: a description size or a field width wrong in both would agree with itself. The typed
//! builders a dependant uses are `crate::fixtures` instead, and they go through the writer on purpose.

use crate::shader_library::shader_blender::ShaderBlender;
use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;
use crate::shader_library::shader_library_file::ShaderLibraryFile;

/// Bytes `CBlender_DESC` occupies, stated as the literal the engine's struct comes to rather than derived from the
/// reader's own constants.
pub(crate) const DESCRIPTION_SIZE: usize = 176;

/// The machine every fixture claims to have been saved on.
pub(crate) const COMPUTER: &str = "OLES";

/// The eight tag bytes as the file stores them.
///
/// `make_clsid` packs the first character into the most significant byte and the description is written whole with a
/// little endian `u64`, so the tag appears reversed on disk. Derived here by reversing the literal, independently of
/// how [`crate::ShaderBlenderClass`] packs it.
pub(crate) fn class_tag(tag: &[u8; 8]) -> [u8; 8] {
  let mut bytes: [u8; 8] = *tag;

  bytes.reverse();

  bytes
}

/// One blender chunk's payload: a description followed by the properties, with nothing between them.
pub(crate) fn blender(tag: &[u8; 8], name: &str, version: u16, properties: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = description(tag, name, version);

  for property in properties {
    bytes.extend_from_slice(property);
  }

  bytes
}

/// `models\model`'s grid as `gamedata/shaders.xr` stores it: the base properties, then the class's own three.
pub(crate) fn model_blender(name: &str, is_alpha_used: bool, reference: i32) -> Vec<u8> {
  blender(
    b"MODEL   ",
    name,
    2,
    &[
      marker("General"),
      integer("Priority", 1, 0, 3),
      boolean("Strict sorting", false),
      marker("Base Texture"),
      text(ShaderBlenderPropertyKind::Texture, "Name", "$base0"),
      text(ShaderBlenderPropertyKind::Matrix, "Transform", "$null"),
      boolean("Use alpha-channel", is_alpha_used),
      integer("Alpha ref", reference, 0, 255),
      token("Tessellation", 0, &[(0, "NO_TESS"), (1, "TESS_PN")]),
    ],
  )
}

/// A library file: the blender chunk holding one numbered child chunk per blender.
pub(crate) fn library(blenders: &[Vec<u8>]) -> Vec<u8> {
  let mut children: Vec<u8> = Vec::new();

  for (index, blender) in blenders.iter().enumerate() {
    children.extend_from_slice(&raw_chunk(index as u32, blender));
  }

  raw_chunk(ShaderLibraryFile::BLENDERS_CHUNK_ID, &children)
}

/// The 176 byte description: class tag, name, computer, save time, version, and the packing padding.
pub(crate) fn description(tag: &[u8; 8], name: &str, version: u16) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(DESCRIPTION_SIZE);

  bytes.extend_from_slice(&class_tag(tag));
  bytes.extend_from_slice(&fixed(name, ShaderBlender::NAME_SIZE));
  bytes.extend_from_slice(&fixed(COMPUTER, ShaderBlender::COMPUTER_SIZE));
  bytes.extend_from_slice(&0u32.to_le_bytes());
  bytes.extend_from_slice(&version.to_le_bytes());
  bytes.extend_from_slice(&[0; 2]);

  assert_eq!(
    bytes.len(),
    DESCRIPTION_SIZE,
    "expect a description of the engine's size"
  );

  bytes
}

pub(crate) fn marker(name: &str) -> Vec<u8> {
  property(ShaderBlenderPropertyKind::Marker, name, &[])
}

pub(crate) fn boolean(name: &str, value: bool) -> Vec<u8> {
  property(ShaderBlenderPropertyKind::Bool, name, &u32::from(value).to_le_bytes())
}

pub(crate) fn integer(name: &str, value: i32, minimum: i32, maximum: i32) -> Vec<u8> {
  let mut payload: Vec<u8> = Vec::new();

  payload.extend_from_slice(&value.to_le_bytes());
  payload.extend_from_slice(&minimum.to_le_bytes());
  payload.extend_from_slice(&maximum.to_le_bytes());

  property(ShaderBlenderPropertyKind::Integer, name, &payload)
}

pub(crate) fn float(name: &str, value: f32, minimum: f32, maximum: f32) -> Vec<u8> {
  let mut payload: Vec<u8> = Vec::new();

  payload.extend_from_slice(&value.to_le_bytes());
  payload.extend_from_slice(&minimum.to_le_bytes());
  payload.extend_from_slice(&maximum.to_le_bytes());

  property(ShaderBlenderPropertyKind::Float, name, &payload)
}

/// One of the five types whose payload is a `string64`.
pub(crate) fn text(kind: ShaderBlenderPropertyKind, name: &str, value: &str) -> Vec<u8> {
  property(kind, name, &fixed(value, ShaderBlenderPropertyKind::TEXT_SIZE))
}

pub(crate) fn token(name: &str, selected: u32, items: &[(u32, &str)]) -> Vec<u8> {
  let mut payload: Vec<u8> = Vec::new();

  payload.extend_from_slice(&selected.to_le_bytes());
  payload.extend_from_slice(&(items.len() as u32).to_le_bytes());

  for (id, label) in items {
    payload.extend_from_slice(&id.to_le_bytes());
    payload.extend_from_slice(&fixed(label, ShaderBlenderPropertyKind::TEXT_SIZE));
  }

  property(ShaderBlenderPropertyKind::Token, name, &payload)
}

pub(crate) fn class_id(name: &str, selected: u64, items: &[u64]) -> Vec<u8> {
  let mut payload: Vec<u8> = Vec::new();

  payload.extend_from_slice(&selected.to_le_bytes());
  payload.extend_from_slice(&(items.len() as u32).to_le_bytes());

  for item in items {
    payload.extend_from_slice(&item.to_le_bytes());
  }

  property(ShaderBlenderPropertyKind::ClassId, name, &payload)
}

/// One property record: its type, a null terminated name, and the payload its type fixes.
pub(crate) fn property(kind: ShaderBlenderPropertyKind, name: &str, payload: &[u8]) -> Vec<u8> {
  raw_property(kind.raw(), name, payload)
}

/// The same, for a type discriminant no reader accepts.
pub(crate) fn raw_property(kind: u32, name: &str, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&kind.to_le_bytes());
  bytes.extend_from_slice(name.as_bytes());
  bytes.push(0);
  bytes.extend_from_slice(payload);

  bytes
}

/// A chunk header and its payload.
pub(crate) fn raw_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// A fixed width field, null padded the way the engine's own buffers are.
fn fixed(value: &str, size: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![0; size];

  bytes[..value.len()].copy_from_slice(value.as_bytes());

  bytes
}
