/// How a pass uses a texture other than as an attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraphTextureAccess {
  /// Sampled or loaded in a shader.
  Sampled,
  StorageRead,
  StorageWrite,
  StorageReadWrite,
  CopySource,
  CopyDestination,
}

impl GraphTextureAccess {
  pub fn is_read(self) -> bool {
    !matches!(self, Self::StorageWrite | Self::CopyDestination)
  }

  pub fn is_write(self) -> bool {
    matches!(
      self,
      Self::StorageWrite | Self::StorageReadWrite | Self::CopyDestination
    )
  }

  /// The usage a texture accessed so needs.
  pub fn get_usage(self) -> wgpu::TextureUsages {
    match self {
      Self::Sampled => wgpu::TextureUsages::TEXTURE_BINDING,
      Self::StorageRead | Self::StorageWrite | Self::StorageReadWrite => wgpu::TextureUsages::STORAGE_BINDING,
      Self::CopySource => wgpu::TextureUsages::COPY_SRC,
      Self::CopyDestination => wgpu::TextureUsages::COPY_DST,
    }
  }
}
