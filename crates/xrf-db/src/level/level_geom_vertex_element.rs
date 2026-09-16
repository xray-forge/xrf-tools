use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Bytes each `D3DDECLTYPE` occupies in a vertex, indexed by the stored type.
///
/// `g_declTypeSizes` (`sdk/include/DirectXMesh/FlexibleVertexFormat.h`). `D3DDECLTYPE_UNUSED` is the one past the
/// end of this table, which is why a declaration's terminator contributes nothing.
const DECLARATION_TYPE_SIZES: [u16; 17] = [4, 8, 12, 16, 4, 4, 4, 8, 4, 4, 8, 4, 8, 4, 4, 4, 8];

/// One element of a vertex buffer's declaration, `D3DVERTEXELEMENT9`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomVertexElement {
  /// Which stream the element is fed from; `0xFF` marks the end of a declaration rather than a stream.
  pub stream: u16,
  /// Where in the vertex the element sits, which is what makes a vertex bigger than the sum of its elements.
  pub offset: u16,
  /// `D3DDECLTYPE`, deciding how many bytes the element occupies.
  pub kind: u8,
  /// `D3DDECLMETHOD`. A tessellator method reads no bytes from the stream at all.
  pub method: u8,
  /// `D3DDECLUSAGE`: position, normal, texture coordinate and so on.
  pub usage: u8,
  pub usage_index: u8,
}

impl LevelGeomVertexElement {
  /// Bytes one element occupies in the file.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + 1 + 1 + 1 + 1;

  /// The stream of `D3DDECL_END`, which terminates a declaration instead of naming a stream.
  pub const TERMINATOR_STREAM: u16 = 0xFF;

  /// `MAXD3DDECLLENGTH`, the most elements a declaration may carry before its terminator.
  pub const MAXIMUM_LENGTH: usize = 64;

  /// `D3DDECLMETHOD_UV`, which the tessellator generates rather than reading, so it occupies no vertex bytes.
  pub const METHOD_UV: u8 = 4;

  /// Whether this element ends the declaration rather than describing anything.
  pub const fn is_terminator(&self) -> bool {
    self.stream == Self::TERMINATOR_STREAM
  }

  /// Bytes the element occupies, or `None` for a type no `D3DDECLTYPE` names.
  pub fn get_kind_size(&self) -> Option<u16> {
    DECLARATION_TYPE_SIZES.get(self.kind as usize).copied()
  }

  /// Whether the element takes its value from the vertex data of `stream` rather than from the tessellator.
  pub const fn is_fed_from(&self, stream: u16) -> bool {
    self.stream == stream && self.method != Self::METHOD_UV
  }
}

impl ChunkReadWrite for LevelGeomVertexElement {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      stream: reader.read_u16::<T>()?,
      offset: reader.read_u16::<T>()?,
      kind: reader.read_u8()?,
      method: reader.read_u8()?,
      usage: reader.read_u8()?,
      usage_index: reader.read_u8()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.stream)?;
    writer.write_u16::<T>(self.offset)?;
    writer.write_u8(self.kind)?;
    writer.write_u8(self.method)?;
    writer.write_u8(self.usage)?;
    writer.write_u8(self.usage_index)?;

    Ok(())
  }
}
