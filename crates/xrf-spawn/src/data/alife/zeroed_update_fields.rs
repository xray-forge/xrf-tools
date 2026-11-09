use xrf_chunk::{ChunkField, ChunkFieldKind, ChunkFieldSource};
use xrf_error::XrfResult;

/// The update packet a spawn packet alone stands for: each field zero of the kind asked, and nothing left, so an
/// optional physics state reads as none. A two-packet class read over it then takes what the engine derives from its
/// spawn packet (`AlifePacketsReadWrite::set_spawned_update`); what stays zero are the members the engine leaves
/// uninitialised (a creature's timestamp and flags, a weapon's zoom, the machine gun's aim), which a root spawn built by
/// GSC's xrAI holds as heap garbage and a rebuild by OpenXRay's as zero.
pub(crate) struct ZeroedUpdateFields;

impl ChunkFieldSource for ZeroedUpdateFields {
  fn read_field(&mut self, kind: ChunkFieldKind) -> XrfResult<ChunkField> {
    Ok(match kind {
      ChunkFieldKind::U8 => ChunkField::U8(0),
      ChunkFieldKind::U16 => ChunkField::U16(0),
      ChunkFieldKind::U32 => ChunkField::U32(0),
      ChunkFieldKind::U64 => ChunkField::U64(0),
      ChunkFieldKind::F32 => ChunkField::F32(0.0),
      ChunkFieldKind::Vector3 => ChunkField::Vector3([0.0; 3]),
      ChunkFieldKind::String => ChunkField::String(String::new()),
    })
  }

  fn is_ended(&self) -> bool {
    true
  }
}
