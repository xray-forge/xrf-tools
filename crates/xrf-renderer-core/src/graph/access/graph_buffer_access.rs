/// How a pass uses a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraphBufferAccess {
  Uniform,
  Vertex,
  Index,
  /// Read as a draw's or a dispatch's arguments.
  Indirect,
  StorageRead,
  StorageWrite,
  StorageReadWrite,
  CopySource,
  CopyDestination,
}

impl GraphBufferAccess {
  pub fn is_read(self) -> bool {
    !matches!(self, Self::StorageWrite | Self::CopyDestination)
  }

  pub fn is_write(self) -> bool {
    matches!(
      self,
      Self::StorageWrite | Self::StorageReadWrite | Self::CopyDestination
    )
  }

  /// The usage a buffer accessed so needs.
  pub fn get_usage(self) -> wgpu::BufferUsages {
    match self {
      Self::Uniform => wgpu::BufferUsages::UNIFORM,
      Self::Vertex => wgpu::BufferUsages::VERTEX,
      Self::Index => wgpu::BufferUsages::INDEX,
      Self::Indirect => wgpu::BufferUsages::INDIRECT,
      Self::StorageRead | Self::StorageWrite | Self::StorageReadWrite => wgpu::BufferUsages::STORAGE,
      Self::CopySource => wgpu::BufferUsages::COPY_SRC,
      Self::CopyDestination => wgpu::BufferUsages::COPY_DST,
    }
  }
}
