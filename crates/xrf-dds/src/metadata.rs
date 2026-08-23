use ddsfile::{D3DFormat, Dds, DxgiFormat};

/// Format identity reported by a DDS header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DdsFormat {
  D3d(D3DFormat),
  Dxgi(DxgiFormat),
  Unknown,
}

/// Header and payload facts needed by DDS consumers without exposing header layout.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DdsMetadata {
  pub file_size: u64,
  pub metadata_size: u64,
  pub data_size: usize,
  pub width: u32,
  pub height: u32,
  pub declared_mipmap_levels: Option<u32>,
  pub mipmap_levels: u32,
  pub minimum_mipmap_size: u32,
  pub depth: Option<u32>,
  pub pitch: Option<u32>,
  pub linear_size: Option<u32>,
  pub block_size: Option<u32>,
  pub bits_per_pixel: Option<u8>,
  pub four_cc: Option<u32>,
  pub has_data_format: bool,
  pub dx10_format: Option<DxgiFormat>,
  pub format: DdsFormat,
}

impl DdsMetadata {
  /// The format's name, keeping an unrecognised format's FourCC visible.
  ///
  /// [`DdsFormat`] is non-exhaustive, so an unrecognised variant reports its FourCC rather than being dropped.
  pub fn get_format_label(&self) -> String {
    match self.format {
      DdsFormat::D3d(format) => format!("{format:?}"),
      DdsFormat::Dxgi(format) => format!("{format:?}"),
      _ => match self.four_cc {
        Some(four_cc) => format!("unknown({four_cc:#010x})"),
        None => String::from("unknown(no fourcc)"),
      },
    }
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
    }
  }
}
