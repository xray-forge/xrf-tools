use xrf_dds::{D3DFormat, DdsFile, DdsFormat, DdsMetadata, DxgiFormat};
use xrf_error::{XrfError, XrfResult};

/// Faces a cube holds.
pub const CUBE_FACES: u32 = 6;

/// A texture read off its file and laid out for upload: its mip chain as the GPU stores it, a cube's six faces each
/// level, or a volume's slices as the layers of its top level alone.
pub struct DecodedTexture {
  pub format: wgpu::TextureFormat,
  pub width: u32,
  pub height: u32,
  /// One, a cube's six faces, or a volume's slices.
  pub layers: u32,
  /// Whether its layers are a volume's slices, which a shader blends between itself.
  pub is_volume: bool,
  /// Each mip level's bytes, largest first, every layer's in turn.
  pub levels: Vec<Vec<u8>>,
  /// The file's own layout.
  pub layout: String,
  /// Whether it was expanded to eight bits a channel rather than kept as stored.
  pub is_expanded: bool,
}

impl DecodedTexture {
  /// Bytes the upload carries, which is what an upload budget spends.
  pub fn get_size(&self) -> u64 {
    self.levels.iter().map(|level| level.len() as u64).sum()
  }

  /// Lays a DDS file out for upload as the engine samples it: block compressed layouts as stored, raw and never
  /// decoded from sRGB, with the file's own mip chain; anything else expanded to eight bits a channel. A cube keeps
  /// its six faces, which only a block layout may hold.
  ///
  /// # Errors
  ///
  /// Returns an error for a file that does not parse, or a layout neither stored nor expandable.
  pub fn from_dds(bytes: &[u8]) -> XrfResult<Self> {
    let file: DdsFile = DdsFile::read_from_bytes(bytes)?;
    let metadata: DdsMetadata = file.metadata();
    let levels: u32 = metadata.mipmap_levels.max(1);

    match to_block_format(&metadata.format) {
      Some(format) if metadata.is_volume => Self::from_volume_blocks(&file, &metadata, format),
      Some(format) if metadata.is_cubemap => Self::from_cube_blocks(&file, &metadata, format, levels),
      None if metadata.is_cubemap => match to_texel_cube_format(&metadata.format) {
        Some(format) => Self::from_cube_blocks(&file, &metadata, format, levels),
        None => Err(XrfError::new_texture_processing_error(format!(
          "A {} cube cannot be expanded",
          metadata.get_format_label()
        ))),
      },
      // A block texture's base must be whole blocks to upload as stored; a smaller one is expanded instead.
      Some(format)
        if metadata.width.is_multiple_of(format.block_dimensions().0)
          && metadata.height.is_multiple_of(format.block_dimensions().1) =>
      {
        Self::from_blocks(&file, &metadata, format, levels)
      }
      _ => Self::from_pixels(&file, &metadata, levels),
    }
  }

  fn from_blocks(file: &DdsFile, metadata: &DdsMetadata, format: wgpu::TextureFormat, levels: u32) -> XrfResult<Self> {
    let (block_width, block_height) = format.block_dimensions();
    let block_bytes: u32 = format
      .block_copy_size(None)
      .ok_or_else(|| XrfError::new_texture_processing_error("A block format without a block size"))?;
    let data: &[u8] = file.get_data();
    let mut offset: usize = 0;
    let mut chain: Vec<Vec<u8>> = Vec::with_capacity(levels as usize);

    for level in 0..levels {
      let width: u32 = (metadata.width >> level).max(1);
      let height: u32 = (metadata.height >> level).max(1);
      let length: usize = (width.div_ceil(block_width) * height.div_ceil(block_height) * block_bytes) as usize;

      // A chain cut short in the file keeps the levels it has.
      let Some(bytes) = data.get(offset..offset + length) else {
        break;
      };

      chain.push(bytes.to_vec());
      offset += length;
    }

    if chain.is_empty() {
      return Err(XrfError::new_texture_processing_error(
        "The texture holds no whole mip level",
      ));
    }

    Ok(Self {
      format,
      width: metadata.width,
      height: metadata.height,
      layers: 1,
      is_volume: false,
      levels: chain,
      layout: metadata.get_format_label().to_string(),
      is_expanded: false,
    })
  }

  /// A volume's slices at its top level, one after another, as layers.
  fn from_volume_blocks(file: &DdsFile, metadata: &DdsMetadata, format: wgpu::TextureFormat) -> XrfResult<Self> {
    let (block_width, block_height) = format.block_dimensions();
    let block_bytes: u32 = format
      .block_copy_size(None)
      .ok_or_else(|| XrfError::new_texture_processing_error("A block format without a block size"))?;
    let slices: u32 = metadata.depth.unwrap_or(1).max(1);
    let slice: usize =
      (metadata.width.div_ceil(block_width) * metadata.height.div_ceil(block_height) * block_bytes) as usize;
    let bytes: &[u8] = file
      .get_data()
      .get(..slice * slices as usize)
      .ok_or_else(|| XrfError::new_texture_processing_error("The volume stops before its last slice ends"))?;

    Ok(Self {
      format,
      width: metadata.width,
      height: metadata.height,
      layers: slices,
      is_volume: true,
      levels: vec![bytes.to_vec()],
      layout: metadata.get_format_label().to_string(),
      is_expanded: false,
    })
  }

  /// A cube's faces, stored one whole chain after another, regrouped by level: each level's faces together, in order.
  /// Texels four bytes each are blocks of one texel here, uploaded as stored.
  fn from_cube_blocks(
    file: &DdsFile,
    metadata: &DdsMetadata,
    format: wgpu::TextureFormat,
    levels: u32,
  ) -> XrfResult<Self> {
    let (block_width, block_height) = format.block_dimensions();
    let block_bytes: u32 = format
      .block_copy_size(None)
      .ok_or_else(|| XrfError::new_texture_processing_error("A block format without a block size"))?;
    let lengths: Vec<usize> = (0..levels)
      .map(|level| {
        let width: u32 = (metadata.width >> level).max(1);
        let height: u32 = (metadata.height >> level).max(1);

        (width.div_ceil(block_width) * height.div_ceil(block_height) * block_bytes) as usize
      })
      .collect();
    let face: usize = lengths.iter().sum();
    let data: &[u8] = file.get_data();

    if data.len() < face * CUBE_FACES as usize {
      return Err(XrfError::new_texture_processing_error(
        "The cube stops before its sixth face ends",
      ));
    }

    let mut offset: usize = 0;
    let chain: Vec<Vec<u8>> = lengths
      .iter()
      .map(|length| {
        let level: Vec<u8> = (0..CUBE_FACES as usize)
          .flat_map(|index| &data[index * face + offset..index * face + offset + length])
          .copied()
          .collect();

        offset += length;

        level
      })
      .collect();

    Ok(Self {
      format,
      width: metadata.width,
      height: metadata.height,
      layers: CUBE_FACES,
      is_volume: false,
      levels: chain,
      layout: metadata.get_format_label().to_string(),
      is_expanded: false,
    })
  }

  fn from_pixels(file: &DdsFile, metadata: &DdsMetadata, levels: u32) -> XrfResult<Self> {
    let chain: Vec<Vec<u8>> = (0..levels)
      .map_while(|level| file.decode_rgba(level).ok().map(|image| image.into_raw()))
      .collect();

    if chain.is_empty() {
      return Err(XrfError::new_texture_processing_error(format!(
        "A {} texture cannot be expanded",
        metadata.get_format_label()
      )));
    }

    Ok(Self {
      format: wgpu::TextureFormat::Rgba8Unorm,
      width: metadata.width,
      height: metadata.height,
      layers: 1,
      is_volume: false,
      levels: chain,
      layout: metadata.get_format_label().to_string(),
      is_expanded: true,
    })
  }
}

/// The GPU format a block compressed layout uploads as, raw; `None` for a layout that is expanded instead.
fn to_block_format(format: &DdsFormat) -> Option<wgpu::TextureFormat> {
  use wgpu::TextureFormat as Gpu;

  Some(match format {
    DdsFormat::D3d(D3DFormat::DXT1) => Gpu::Bc1RgbaUnorm,
    DdsFormat::D3d(D3DFormat::DXT2 | D3DFormat::DXT3) => Gpu::Bc2RgbaUnorm,
    DdsFormat::D3d(D3DFormat::DXT4 | D3DFormat::DXT5) => Gpu::Bc3RgbaUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB | DxgiFormat::BC1_Typeless) => Gpu::Bc1RgbaUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB | DxgiFormat::BC2_Typeless) => Gpu::Bc2RgbaUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB | DxgiFormat::BC3_Typeless) => Gpu::Bc3RgbaUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC4_UNorm | DxgiFormat::BC4_Typeless) => Gpu::Bc4RUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC4_SNorm) => Gpu::Bc4RSnorm,
    DdsFormat::Dxgi(DxgiFormat::BC5_UNorm | DxgiFormat::BC5_Typeless) => Gpu::Bc5RgUnorm,
    DdsFormat::Dxgi(DxgiFormat::BC5_SNorm) => Gpu::Bc5RgSnorm,
    DdsFormat::Dxgi(DxgiFormat::BC6H_UF16 | DxgiFormat::BC6H_Typeless) => Gpu::Bc6hRgbUfloat,
    DdsFormat::Dxgi(DxgiFormat::BC6H_SF16) => Gpu::Bc6hRgbFloat,
    DdsFormat::Dxgi(DxgiFormat::BC7_UNorm | DxgiFormat::BC7_UNorm_sRGB | DxgiFormat::BC7_Typeless) => Gpu::Bc7RgbaUnorm,
    _ => return None,
  })
}

/// The GPU format a cube of texels four bytes each uploads as, raw, the way an irradiance cube is stored; `None` for any
/// other texel layout.
fn to_texel_cube_format(format: &DdsFormat) -> Option<wgpu::TextureFormat> {
  Some(match format {
    DdsFormat::D3d(D3DFormat::A8R8G8B8 | D3DFormat::X8R8G8B8) => wgpu::TextureFormat::Bgra8Unorm,
    DdsFormat::D3d(D3DFormat::A8B8G8R8 | D3DFormat::X8B8G8R8) => wgpu::TextureFormat::Rgba8Unorm,
    DdsFormat::Dxgi(DxgiFormat::B8G8R8A8_UNorm | DxgiFormat::B8G8R8X8_UNorm) => wgpu::TextureFormat::Bgra8Unorm,
    DdsFormat::Dxgi(DxgiFormat::R8G8B8A8_UNorm) => wgpu::TextureFormat::Rgba8Unorm,
    _ => return None,
  })
}
