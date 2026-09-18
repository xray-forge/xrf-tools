use xrf_spawn::XRayByteOrder;
use xrf_thm::{
  ThmBumpChunk, ThmBumpMode, ThmDetailChunk, ThmFile, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType,
};

/// A descriptor file, built by changing one thing at a time about the one the SDK writes for a plain image.
#[derive(Clone, Debug)]
pub struct ThmFixture {
  pub file: ThmFile,
}

impl ThmFixture {
  /// The SDK's default virtual height, `STextureParams::Clear` (`ETextureParams.h`).
  pub const DEFAULT_VIRTUAL_HEIGHT: f32 = ThmBumpChunk::DEFAULT_VIRTUAL_HEIGHT;

  /// A plain 2D image descriptor: the SDK's own defaults for a new file, sized as a 512 square.
  pub fn image() -> Self {
    Self {
      file: ThmFile {
        texture_param: Some(ThmTextureParamChunk {
          width: 512,
          height: 512,
          ..ThmTextureParamChunk::default()
        }),
        ..ThmFile::new_texture()
      },
    }
  }

  pub fn with_texture_type(mut self, texture_type: ThmTextureType) -> Self {
    self.file.texture_type = Some(texture_type);
    self
  }

  pub fn without_texture_type(mut self) -> Self {
    self.file.texture_type = None;
    self
  }

  pub fn without_texture_param(mut self) -> Self {
    self.file.texture_param = None;
    self
  }

  /// A bump declaration in `mode` naming `name`, at the default virtual height.
  pub fn with_bump(mut self, mode: ThmBumpMode, name: &str) -> Self {
    self.file.bump = Some(ThmBumpChunk {
      mode,
      name: name.to_owned(),
      ..ThmBumpChunk::default()
    });
    self
  }

  pub fn with_virtual_height(mut self, virtual_height: f32) -> Self {
    if let Some(bump) = self.file.bump.as_mut() {
      bump.virtual_height = virtual_height;
    }

    self
  }

  pub fn without_bump(mut self) -> Self {
    self.file.bump = None;
    self
  }

  /// A detail association with the flags that decide whether the engine applies it.
  pub fn with_detail(mut self, name: &str, scale: f32, flags: &[ThmTextureFlag]) -> Self {
    self.file.detail = Some(ThmDetailChunk {
      name: name.to_owned(),
      scale,
    });

    if let Some(param) = self.file.texture_param.as_mut() {
      for flag in flags {
        param.flags.set(*flag, true);
      }
    }

    self
  }

  /// The file's bytes, in the engine's byte order.
  ///
  /// # Panics
  ///
  /// Never for a fixture of sane size: the only failure a chunk writer can report is a payload past `u32::MAX`.
  pub fn to_bytes(&self) -> Vec<u8> {
    self
      .file
      .write_to_bytes::<XRayByteOrder>()
      .expect("fixture descriptor is writable")
  }
}
