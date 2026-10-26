/// The Direct3D flexible vertex format a static visual's `OGF_VERTICES` declares, which `Fvisual::Load` sizes with
/// `GetFVFVertexSize` (`xray-16/sdk/include/DirectXMesh/FlexibleVertexFormat.h`, `ComputeVertexSize`).
///
/// Only the `D3DFVF_XYZ` position is decoded: it is what the SDK's object exporter writes
/// (`D3DFVF_XYZ | D3DFVF_NORMAL | D3DFVF_TEX1`, `ExportObjectOGF.cpp`), and the transformed and blended positions are
/// not a static visual's. Fields follow the position in declaration order: normal, point size, diffuse, specular, then
/// each texture coordinate set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfFvf {
  pub raw: u32,
  pub has_normal: bool,
  pub has_point_size: bool,
  pub has_diffuse: bool,
  pub has_specular: bool,
  /// Floats each texture coordinate set holds, one to four.
  pub texture_coordinate_sizes: Vec<usize>,
}

impl OgfFvf {
  const POSITION_MASK: u32 = 0x400E;
  const XYZ: u32 = 0x002;
  const NORMAL: u32 = 0x010;
  const POINT_SIZE: u32 = 0x020;
  const DIFFUSE: u32 = 0x040;
  const SPECULAR: u32 = 0x080;
  const TEXTURE_COUNT_MASK: u32 = 0xF00;
  const TEXTURE_COUNT_SHIFT: u32 = 8;
  /// `D3DFVF_RESERVED0 | D3DFVF_RESERVED2` outside the position bits, which `ComputeVertexSize` refuses.
  const RESERVED: u32 = 0x6001 & !Self::POSITION_MASK;
  const MAXIMUM_TEXTURE_SETS: usize = 8;

  /// The format a raw code names, or `None` for one this does not decode.
  pub fn from_raw(raw: u32) -> Option<Self> {
    let texture_sets: usize = ((raw & Self::TEXTURE_COUNT_MASK) >> Self::TEXTURE_COUNT_SHIFT) as usize;

    if raw & Self::RESERVED != 0 || raw & Self::POSITION_MASK != Self::XYZ || texture_sets > Self::MAXIMUM_TEXTURE_SETS
    {
      return None;
    }

    Some(Self {
      raw,
      has_normal: raw & Self::NORMAL != 0,
      has_point_size: raw & Self::POINT_SIZE != 0,
      has_diffuse: raw & Self::DIFFUSE != 0,
      has_specular: raw & Self::SPECULAR != 0,
      texture_coordinate_sizes: (0..texture_sets)
        .map(|set| match (raw >> (16 + 2 * set)) & 3 {
          0 => 2,
          1 => 3,
          2 => 4,
          _ => 1,
        })
        .collect(),
    })
  }

  /// Bytes one vertex occupies, as `ComputeVertexSize` counts them.
  pub fn get_stride(&self) -> usize {
    3 * 4
      + usize::from(self.has_normal) * 3 * 4
      + usize::from(self.has_point_size) * 4
      + usize::from(self.has_diffuse) * 4
      + usize::from(self.has_specular) * 4
      + self.texture_coordinate_sizes.iter().sum::<usize>() * 4
  }
}
