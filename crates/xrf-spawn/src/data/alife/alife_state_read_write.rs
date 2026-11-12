use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxRead, LtxWrite};

use crate::data::alife::spawn_layout::SpawnLayout;

/// A server class payload whose spawn fields depend on the object's [`SpawnLayout`], read and written as its
/// `STATE_Read` and `STATE_Write` do at that layout's versions.
///
/// A field the layout lacks reads as the value the engine holds in its place, is not written, and refuses to write any
/// other value rather than drop it; the LTX form leaves out the same keys and imports the same values, so bytes, fields
/// and text agree. A class whose spawn fields are the same at every supported version stays on `ChunkFieldReadWrite`.
pub(crate) trait AlifeStateReadWrite: Sized {
  /// Read the payload's spawn fields as `layout` lays them out.
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self>;

  /// Write the payload's spawn fields as `layout` lays them out.
  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult;

  /// Import the payload from its LTX section, the keys `layout` carries.
  fn import_state<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self>;

  /// Export the payload into its LTX section, the keys `layout` carries.
  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult;
}
