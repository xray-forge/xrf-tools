use serde::Serialize;
use xrf_level::LevelGeomVertexBuffer;

/// One vertex declaration and how much of a level's geometry is stored in it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexLayoutReport {
  pub declaration: String,
  /// Bytes one vertex occupies, absent when the declaration names a type `D3DDECLTYPE` does not.
  pub stride: Option<u32>,
  pub buffers: usize,
  pub vertices: u64,
}

impl LevelVertexLayoutReport {
  /// `D3DDECLUSAGE` by value; an index past the end is a usage the enumeration does not name.
  const USAGES: [&'static str; 14] = [
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

  /// `D3DDECLTYPE` by value, in the order `g_declTypeSizes` sizes them.
  const KINDS: [&'static str; 17] = [
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

  /// A declaration as one line, so two buffers sharing a layout collapse into one reported row.
  pub fn describe(buffer: &LevelGeomVertexBuffer) -> String {
    buffer
      .declaration
      .iter()
      .map(|element| {
        format!(
          "{}{}:{}",
          Self::describe_usage(element.usage),
          element.usage_index,
          Self::describe_kind(element.kind)
        )
      })
      .collect::<Vec<_>>()
      .join(" | ")
  }

  fn describe_usage(usage: u8) -> String {
    Self::USAGES
      .get(usage as usize)
      .map_or_else(|| format!("USAGE({usage})"), |it| String::from(*it))
  }

  fn describe_kind(kind: u8) -> String {
    Self::KINDS
      .get(kind as usize)
      .map_or_else(|| format!("TYPE({kind})"), |it| String::from(*it))
  }
}
