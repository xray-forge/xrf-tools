use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_db::{ThmBumpChunk, ThmDetailChunk, ThmTextureParamChunk, ThmTextureTypeChunk};

/// A descriptor file assembled chunk by chunk, in the order `STextureParams::Save` writes them.
///
/// Starts as the descriptor the SDK writes for a plain image with no bump and no detail, and each `with_*` changes one
/// thing, so a test reads as the difference from that baseline. An `Option` left `None` writes no chunk at all, which
/// is how the absent-chunk states are produced.
#[derive(Clone, Debug)]
pub struct ThmFixture {
  pub texture_param: Option<ThmTextureParamChunk>,
  pub texture_type: Option<ThmTextureTypeChunk>,
  pub detail: Option<ThmDetailChunk>,
  pub bump: Option<ThmBumpChunk>,
  /// Bytes written under an id this crate never reads, standing in for the authoring chunks a real file carries.
  pub opaque: Option<(u32, Vec<u8>)>,
}

impl ThmFixture {
  /// The SDK's default virtual height, `STextureParams::Clear` (`ETextureParams.h:142`).
  pub const DEFAULT_VIRTUAL_HEIGHT: f32 = 0.05;

  /// A plain 2D image descriptor: default params, image type, no detail, bump mode `none` with an empty name.
  pub fn image() -> Self {
    Self {
      texture_param: Some(ThmTextureParamChunk {
        format: 0,
        flags: 0,
        border_color: 0,
        fade_color: 0,
        fade_amount: 0,
        mip_filter: 0,
        width: 512,
        height: 512,
      }),
      texture_type: Some(ThmTextureTypeChunk {
        texture_type: ThmTextureTypeChunk::IMAGE,
      }),
      detail: None,
      bump: Some(ThmBumpChunk {
        virtual_height: Self::DEFAULT_VIRTUAL_HEIGHT,
        mode: ThmBumpChunk::MODE_NONE,
        name: String::new(),
      }),
      opaque: Some((0x0819, vec![0])),
    }
  }

  pub fn with_texture_type(mut self, texture_type: u32) -> Self {
    self.texture_type = Some(ThmTextureTypeChunk { texture_type });
    self
  }

  pub fn without_texture_type(mut self) -> Self {
    self.texture_type = None;
    self
  }

  pub fn without_texture_param(mut self) -> Self {
    self.texture_param = None;
    self
  }

  /// A bump declaration in `mode` naming `name`, at the default virtual height.
  pub fn with_bump(mut self, mode: u32, name: &str) -> Self {
    self.bump = Some(ThmBumpChunk {
      virtual_height: Self::DEFAULT_VIRTUAL_HEIGHT,
      mode,
      name: name.to_owned(),
    });
    self
  }

  pub fn with_virtual_height(mut self, virtual_height: f32) -> Self {
    if let Some(bump) = self.bump.as_mut() {
      bump.virtual_height = virtual_height;
    }

    self
  }

  pub fn without_bump(mut self) -> Self {
    self.bump = None;
    self
  }

  /// A detail association with the given texture param flags, which is what decides whether it is live.
  pub fn with_detail(mut self, name: &str, scale: f32, flags: u32) -> Self {
    self.detail = Some(ThmDetailChunk {
      name: name.to_owned(),
      scale,
    });

    if let Some(param) = self.texture_param.as_mut() {
      param.flags |= flags;
    }

    self
  }

  /// The file's bytes, in the engine's byte order.
  ///
  /// # Panics
  ///
  /// Never for a fixture of sane size: the only failure a chunk writer can report is a payload past `u32::MAX`.
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();

    if let Some(param) = &self.texture_param {
      Self::push_chunk(&mut buffer, ThmTextureParamChunk::CHUNK_ID, param);
    }

    if let Some(texture_type) = &self.texture_type {
      Self::push_chunk(&mut buffer, ThmTextureTypeChunk::CHUNK_ID, texture_type);
    }

    if let Some(detail) = &self.detail {
      Self::push_chunk(&mut buffer, ThmDetailChunk::CHUNK_ID, detail);
    }

    if let Some(bump) = &self.bump {
      Self::push_chunk(&mut buffer, ThmBumpChunk::CHUNK_ID, bump);
    }

    if let Some((id, payload)) = &self.opaque {
      let mut writer: ChunkWriter = ChunkWriter::new();

      writer.buffer.extend_from_slice(payload);
      writer
        .flush_chunk_into::<XRayByteOrder>(&mut buffer, *id)
        .expect("opaque fixture chunk is writable");
    }

    buffer
  }

  fn push_chunk<C: ChunkReadWrite>(buffer: &mut Vec<u8>, id: u32, chunk: &C) {
    let mut writer: ChunkWriter = ChunkWriter::new();

    chunk
      .write::<XRayByteOrder>(&mut writer)
      .expect("fixture chunk is writable");
    writer
      .flush_chunk_into::<XRayByteOrder>(buffer, id)
      .expect("fixture chunk is framable");
  }
}
