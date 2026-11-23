use crate::scene::texture::sequence::decoded_sequence::DecodedSequence;
use crate::scene::texture::sequence::texture_sequence_timing::TextureSequenceTiming;

/// A sequence on the GPU: its frames as one array, and the texture its slot samples, into which the frame due is copied
/// when the frame changes. The engine binds each frame's own texture in turn; copying into one keeps the slot, so no
/// bind group is made again and every surface, map and particle naming it plays it.
pub(crate) struct PlayedSequence {
  frames: wgpu::Texture,
  shown: wgpu::Texture,
  count: u32,
  timing: TextureSequenceTiming,
  /// The frame `shown` holds.
  current: u32,
}

impl PlayedSequence {
  /// Uploads a sequence's frames, its first shown, and answers it with the view its slot binds.
  pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sequence: &DecodedSequence,
  ) -> (Self, wgpu::TextureView) {
    let frames: wgpu::Texture = sequence.frames.upload_layers(
      device,
      queue,
      "texture sequence frames",
      sequence.get_count(),
      wgpu::TextureUsages::COPY_SRC,
    );
    let shown: wgpu::Texture =
      sequence
        .frames
        .upload_layers(device, queue, "level texture", 1, wgpu::TextureUsages::empty());
    let view: wgpu::TextureView = shown.create_view(&wgpu::TextureViewDescriptor {
      dimension: Some(wgpu::TextureViewDimension::D2),
      ..Default::default()
    });

    (
      Self {
        frames,
        shown,
        count: sequence.get_count(),
        timing: sequence.timing,
        current: 0,
      },
      view,
    )
  }

  /// Copies the frame due `elapsed_ms` into the clock into the shown texture, every level of it, where another shows.
  pub(crate) fn play(&mut self, encoder: &mut wgpu::CommandEncoder, elapsed_ms: u64) {
    let frame: u32 = self.timing.get_frame(elapsed_ms, self.count);

    if frame == self.current {
      return;
    }

    self.current = frame;

    let size: wgpu::Extent3d = self.shown.size();

    for level in 0..self.shown.mip_level_count() {
      encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
          texture: &self.frames,
          mip_level: level,
          origin: wgpu::Origin3d { x: 0, y: 0, z: frame },
          aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
          texture: &self.shown,
          mip_level: level,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
          depth_or_array_layers: 1,
          ..size
            .mip_level_size(level, wgpu::TextureDimension::D2)
            .physical_size(self.shown.format())
        },
      );
    }
  }

  /// The frame its slot shows.
  #[cfg(test)]
  pub(crate) fn get_current(&self) -> u32 {
    self.current
  }
}
