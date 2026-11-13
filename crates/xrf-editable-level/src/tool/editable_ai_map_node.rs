use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::AiNodePosition;

/// One node of the AI map tool, `SAINode` as `SAINode::SaveStream` writes it: its four neighbours, its plane, its cell
/// and height packed against the tool's box, and the editor's flags for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditableAiMapNode {
  /// The neighbours across each edge, by node index, [`Self::NO_LINK`] for none.
  pub links: [u32; 4],
  /// The plane normal, compressed (`pvCompress`).
  pub plane: u16,
  /// The cell, and the height across the tool's box.
  pub position: AiNodePosition,
  /// `SAINode`'s flags: selected and the like.
  pub flags: u8,
}

impl EditableAiMapNode {
  /// `InvalidNode` as the three bytes a link is stored in.
  pub const NO_LINK: u32 = 0x00FF_FFFF;

  /// Bytes a node occupies: four three-byte links, the plane, the position and the flags.
  pub const SERIALIZED_SIZE: u64 = 4 * 3 + 2 + 6 + 1;

  pub(crate) fn read<T: ByteOrder>(reader: &mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<Self> {
    let mut links: [u32; 4] = [0; 4];

    for link in &mut links {
      *link = reader.read_u24::<T>()?;
    }

    Ok(Self {
      links,
      plane: reader.read_u16::<T>()?,
      position: reader.read_xr::<T, _>()?,
      flags: reader.read_u8()?,
    })
  }

  pub(crate) fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for link in self.links {
      if link > Self::NO_LINK {
        return Err(XrfError::new_invalid_error(format!(
          "AI map link {link} does not fit the three bytes the tool gives it"
        )));
      }

      writer.write_u24::<T>(link)?;
    }

    writer.write_u16::<T>(self.plane)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_u8(self.flags)?;

    Ok(())
  }
}
