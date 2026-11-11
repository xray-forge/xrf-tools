use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

/// One node of the AI map tool, `SAINode` as `SAINode::SaveStream` writes it: its four neighbours, its plane, its cell
/// and height packed against the tool's box (`NodePosition`), and the editor's flags for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditableAiMapNode {
  /// The neighbours across each edge, by node index, [`Self::NO_LINK`] for none.
  pub links: [u32; 4],
  /// The plane normal, compressed (`pvCompress`).
  pub plane: u16,
  /// The cell along x and z, in patches from the world origin.
  pub x: i16,
  pub z: i16,
  /// The height across the tool's box, 0 at its floor and 65535 at its top.
  pub y: u16,
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
      x: reader.read_i16::<T>()?,
      y: reader.read_u16::<T>()?,
      z: reader.read_i16::<T>()?,
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
    writer.write_i16::<T>(self.x)?;
    writer.write_u16::<T>(self.y)?;
    writer.write_i16::<T>(self.z)?;
    writer.write_u8(self.flags)?;

    Ok(())
  }
}
