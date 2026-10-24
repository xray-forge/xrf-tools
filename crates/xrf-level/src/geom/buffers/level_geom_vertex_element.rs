use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Bytes each `D3DDECLTYPE` occupies in a vertex, indexed by the stored type.
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

  /// `D3DDECLUSAGE` names by value.
  const USAGE_NAMES: [&'static str; 14] = [
    "POSITION",
    "BLENDWEIGHT",
    "BLENDINDICES",
    "NORMAL",
    "PSIZE",
    "TEXCOORD",
    "TANGENT",
    "BINORMAL",
    "TESSFACTOR",
    "POSITIONT",
    "COLOR",
    "FOG",
    "DEPTH",
    "SAMPLE",
  ];

  /// `D3DDECLTYPE` names by value, in the order `DECLARATION_TYPE_SIZES` sizes them.
  const KIND_NAMES: [&'static str; 17] = [
    "FLOAT1",
    "FLOAT2",
    "FLOAT3",
    "FLOAT4",
    "D3DCOLOR",
    "UBYTE4",
    "SHORT2",
    "SHORT4",
    "UBYTE4N",
    "SHORT2N",
    "SHORT4N",
    "USHORT2N",
    "USHORT4N",
    "UDEC3",
    "DEC3N",
    "FLOAT16_2",
    "FLOAT16_4",
  ];

  /// Its `D3DDECLUSAGE` by name, or `USAGE(<value>)` for one the enumeration does not name.
  pub fn get_usage_name(&self) -> String {
    Self::USAGE_NAMES
      .get(self.usage as usize)
      .map_or_else(|| format!("USAGE({})", self.usage), |name| String::from(*name))
  }

  /// Its `D3DDECLTYPE` by name, or `TYPE(<value>)` for one the enumeration does not name.
  pub fn get_kind_name(&self) -> String {
    Self::KIND_NAMES
      .get(self.kind as usize)
      .map_or_else(|| format!("TYPE({})", self.kind), |name| String::from(*name))
  }

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
