use ddsfile::{Caps2, Dds, DxgiFormat};

use crate::file::dds_format::DdsFormat;

/// Header and payload facts needed by DDS consumers without exposing header layout.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DdsMetadata {
  /// Bytes of the whole file.
  pub file_size: u64,
  /// Bytes of the header, magic included: 128, or 148 with the DX10 extension.
  pub metadata_size: u64,
  /// Bytes of the payload, every layer and level of it.
  pub data_size: usize,
  pub width: u32,
  pub height: u32,
  /// The mip count the header states, or `None` where it states none.
  pub declared_mipmap_levels: Option<u32>,
  /// Mip levels the file carries, base included: the stated count, or one where none is stated.
  pub mipmap_levels: u32,
  /// Bytes the smallest level of this format occupies: one block for a compressed format, one pixel otherwise.
  pub minimum_mipmap_size: u32,
  /// Slices of a volume, as the header states them.
  pub depth: Option<u32>,
  /// Bytes per row of the base level, for an uncompressed header that states it.
  pub pitch: Option<u32>,
  /// Bytes of the base level, for a compressed header that states it.
  pub linear_size: Option<u32>,
  /// Bytes per block of a block-compressed format.
  pub block_size: Option<u32>,
  /// Bits per pixel of an uncompressed format.
  pub bits_per_pixel: Option<u8>,
  /// The FourCC code the pixel format carries.
  pub four_cc: Option<u32>,
  /// Whether `ddsfile` recognises the format at all, which [`Self::block_size`] and [`Self::bits_per_pixel`] need.
  pub has_data_format: bool,
  /// The format a DX10 header extension names.
  pub dx10_format: Option<DxgiFormat>,
  pub format: DdsFormat,
  /// Whether it holds a cube's six faces, each its own mip chain in turn, `+x -x +y -y +z -z`.
  pub is_cubemap: bool,
  /// Whether it holds a volume's slices, each level's slices one after another.
  pub is_volume: bool,
}

impl DdsMetadata {
  /// The format's name, keeping an unrecognised format's FourCC visible.
  pub fn get_format_label(&self) -> String {
    match self.format {
      DdsFormat::D3d(format) => format!("{format:?}"),
      DdsFormat::Dxgi(format) => format!("{format:?}"),
      DdsFormat::Unknown => match self.four_cc {
        Some(four_cc) => format!("unknown({four_cc:#010x})"),
        None => String::from("unknown(no fourcc)"),
      },
    }
  }

  /// The largest mip level whose longer side holds `side` texels at most, or the smallest the file carries where none
  /// does: what a measure decodes when a coarser level reads the same statistics at a fraction of the cost.
  pub fn get_level_within(&self, side: u32) -> u32 {
    let largest: u32 = self.width.max(self.height).max(1);
    let within: u32 = largest.div_ceil(side.max(1)).next_power_of_two().ilog2();

    within.min(self.mipmap_levels.max(1) - 1)
  }

  pub(crate) fn from_dds(dds: &Dds, file_size: u64, metadata_size: u64) -> Self {
    let data_format = dds.get_format();
    let format: DdsFormat = if let Some(format) = dds.get_d3d_format() {
      DdsFormat::D3d(format)
    } else if let Some(format) = dds.get_dxgi_format() {
      DdsFormat::Dxgi(format)
    } else {
      DdsFormat::Unknown
    };

    Self {
      file_size,
      metadata_size,
      data_size: dds.data.len(),
      width: dds.header.width,
      height: dds.header.height,
      declared_mipmap_levels: dds.header.mip_map_count,
      mipmap_levels: dds.get_num_mipmap_levels(),
      minimum_mipmap_size: dds.get_min_mipmap_size_in_bytes(),
      depth: dds.header.depth,
      pitch: dds.header.pitch,
      linear_size: dds.header.linear_size,
      block_size: data_format.as_ref().and_then(|format| format.get_block_size()),
      bits_per_pixel: data_format.as_ref().and_then(|format| format.get_bits_per_pixel()),
      four_cc: data_format
        .as_ref()
        .and_then(|format| format.get_fourcc())
        .map(|four_cc| four_cc.0),
      has_data_format: data_format.is_some(),
      dx10_format: dds.header10.as_ref().map(|header| header.dxgi_format),
      format,
      is_cubemap: dds.header.caps2.contains(Caps2::CUBEMAP),
      is_volume: dds.header.caps2.contains(Caps2::VOLUME),
    }
  }
}
