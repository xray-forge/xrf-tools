//! Visuals carrying the malformed shapes issue 0014 is about, and the well-formed one they are judged against.

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::chunks::ogf_header_chunk::OgfHeaderChunk;
use crate::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::data::ogf_box::OgfBox;
use crate::data::ogf_sphere::OgfSphere;

/// The four references shipped Anomaly zombied faces declare.
pub(crate) const DECLARED_REFS: [&str; 4] = [
  "actors\\stalker_animation",
  "actors\\stalker_scripts_animation",
  "actors\\zombied_animation",
  "actors\\stalker_smart_cover_animation",
];

/// The fifth reference those faces carry without counting it, which the engine therefore never loads.
pub(crate) const SPLIT_REF: &str = "actors\\stalker_scenario_animation";

/// Where the declared chunk size cuts that fifth reference, leaving `actors\st` inside the chunk.
pub(crate) const SPLIT_AT: usize = 9;

/// Lay out one chunk the way the format does: id, payload length, then the payload.
fn raw_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// A chunk header declaring a payload the file does not contain.
fn raw_oversized_chunk(id: u32, declared: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&declared.to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// The header chunk every visual needs before anything else can be read out of it.
pub(crate) fn header_bytes() -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
    version: 4,
    model_type: 3,
    shader_id: 0,
    bounding_box: OgfBox {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    bounding_sphere: OgfSphere {
      position: Vector3d::new(0.0, 0.0, 0.0),
      radius: 1.0,
    },
  })?;

  writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfHeaderChunk::CHUNK_ID)
}

/// Count, then that many NUL-terminated paths.
pub(crate) fn kinematics_payload(refs: &[&str]) -> Vec<u8> {
  let mut payload: Vec<u8> = (refs.len() as u32).to_le_bytes().to_vec();

  for motion_ref in refs {
    payload.extend_from_slice(motion_ref.as_bytes());
    payload.push(0);
  }

  payload
}

/// A well-formed visual: header, then a motion refs chunk whose declared size matches its payload.
pub(crate) fn well_formed() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(
    OgfKinematicsChunk::CHUNK_ID,
    &kinematics_payload(&DECLARED_REFS),
  ));

  Ok(bytes)
}

/// The Anomaly zombied face shape: a fifth reference split across the declared bounds of chunk 24.
pub(crate) fn split_motion_ref() -> XrfResult<Vec<u8>> {
  let mut full: Vec<u8> = SPLIT_REF.as_bytes().to_vec();

  full.push(0);

  let mut payload: Vec<u8> = kinematics_payload(&DECLARED_REFS);

  payload.extend_from_slice(&full[..SPLIT_AT]);

  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(OgfKinematicsChunk::CHUNK_ID, &payload));
  bytes.extend_from_slice(&full[SPLIT_AT..]);

  Ok(bytes)
}

/// The `wpn_m1891` shape: a complete visual followed by two stray bytes.
pub(crate) fn trailing_fragment() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = well_formed()?;

  bytes.extend_from_slice(b"\r\n");

  Ok(bytes)
}

/// The `wpn_gp100` shape: a chunk declaring far more than the file holds.
pub(crate) fn truncated_chunk() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_oversized_chunk(23, 4343, b"dynamics\\weapons\\wpn_gp100_lod"));

  Ok(bytes)
}

/// Trailing bytes long enough to be a header and belonging to nothing, which nothing may accept.
pub(crate) fn unexplained_residue() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = well_formed()?;

  bytes.extend_from_slice(b"alker_scenario_animation\0");

  Ok(bytes)
}

/// The reference the `conserva_hud` shape cuts.
pub(crate) const COUNTED_REF: &str = "gwr\\food\\conserva\\conserva_hud_animation";

/// The `conserva_hud` shape: the declared size cuts a reference the count *does* govern.
pub(crate) fn split_counted_ref() -> XrfResult<Vec<u8>> {
  spilled_refs(&[COUNTED_REF], kinematics_payload(&[COUNTED_REF]).len() - 14)
}

/// The SGM `wpn_hand` shape: counted references running on past the declared size, `declared` bytes of the payload
/// inside the chunk and the rest after it, to the end of the file.
pub(crate) fn spilled_refs(refs: &[&str], declared: usize) -> XrfResult<Vec<u8>> {
  let payload: Vec<u8> = kinematics_payload(refs);
  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(OgfKinematicsChunk::CHUNK_ID, &payload[..declared]));
  bytes.extend_from_slice(&payload[declared..]);

  Ok(bytes)
}

/// A well-formed visual whose motion refs chunk holds exactly `refs`.
pub(crate) fn with_refs(refs: &[&str]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(OgfKinematicsChunk::CHUNK_ID, &kinematics_payload(refs)));

  Ok(bytes)
}

/// The legacy motion refs chunk, one comma-joined string, ending the file with a size four bytes larger than it is:
/// the shape GAMMA's weapon HUDs ship in.
pub(crate) fn overstated_last_chunk() -> XrfResult<(Vec<u8>, Vec<u8>)> {
  let payload: &[u8] = b"anomaly_weapons\\hud_animation\\wpn_axe_hud_animation\0";
  let mut overstated: Vec<u8> = header_bytes()?;
  let mut exact: Vec<u8> = overstated.clone();

  overstated.extend(raw_oversized_chunk(
    OgfKinematicsChunk::CHUNK_ID_OLD,
    payload.len() as u32 + 4,
    payload,
  ));
  exact.extend(raw_chunk(OgfKinematicsChunk::CHUNK_ID_OLD, payload));

  Ok((overstated, exact))
}

/// A well-formed visual whose header chunk id carries the compression mark, the protection some mods apply: the
/// engine reads the header with `r_chunk_safe`, which masks the mark off and reads the bytes as stored.
pub(crate) fn marked_header() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = well_formed()?;

  bytes[3] |= 0x80;

  Ok(bytes)
}

/// A visual whose user data chunk carries the compression mark, which the engine would decompress as it opens it.
pub(crate) fn marked_opened_chunk() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = well_formed()?;

  bytes.extend(raw_chunk(17 | (1 << 31), b"[section]"));

  Ok(bytes)
}
