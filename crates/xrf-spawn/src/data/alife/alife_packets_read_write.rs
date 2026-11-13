use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
#[cfg(test)]
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::XrfResult;
use xrf_ltx::{LtxRead, LtxWrite};

use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// A server class payload whose update packet is not empty, read and written across both of an object's packets as
/// the object's [`SpawnLayout`] lays them out.
///
/// Each class takes its spawn fields from the spawn packet and its update fields from the update packet, after its
/// base's in each, as its `STATE_Read` and `UPDATE_Read` do. A class whose update packet is empty, and every base it
/// inherits, reads the spawn packet alone through `ChunkFieldReadWrite`, or through `AlifeStateReadWrite` where its
/// fields depend on the layout. Either packet is bytes or fields, as the engine's `NET_Packet` is with or without an ini
/// stream. A spawn field the layout lacks follows `AlifeStateReadWrite`'s rule, in bytes and text alike.
pub(crate) trait AlifePacketsReadWrite: Sized {
  /// Read the payload's spawn fields from `spawn` and its update fields from `update`, laid out as `layout`.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self>;

  /// Write the payload's spawn fields into `spawn` and its update fields into `update`, laid out as `layout`.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult;

  /// Import the payload from its LTX section, the keys `layout` carries.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self>;

  /// Export the payload into its LTX section, the keys `layout` carries.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult;

  /// Set the update fields as the server entity holds them right after `Spawn_Read`, the state xrAI writes into a root
  /// spawn's update packet: what `STATE_Read` takes from the spawn packet, and the constructor's values. Read first
  /// from [`crate::data::alife::zeroed_update_fields::ZeroedUpdateFields`], every field the engine leaves
  /// uninitialised stays zero, as a clean rebuild writes it.
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader);

  /// Write the payload at `layout` and read it back, refusing a byte of either packet left unread, with the sizes of
  /// the spawn and update fields written.
  #[cfg(test)]
  fn read_back<T: ByteOrder>(&self, layout: SpawnLayout) -> XrfResult<(usize, usize, Self)> {
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    self.write::<T, _, _>(&mut spawn, &mut update, layout)?;

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;
    let read: Self = Self::read::<T, _, _>(&mut spawn_reader, &mut update_reader, layout)?;

    spawn_reader.assert_read("Expect the spawn fields to be read whole")?;
    update_reader.assert_read("Expect the update fields to be read whole")?;

    Ok((spawn.buffer.len(), update.buffer.len(), read))
  }
}
