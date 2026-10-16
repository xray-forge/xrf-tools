use std::collections::HashMap;

/// Views of one mip and layer of the textures render passes draw into, kept across frames by the texture they view: a
/// view no frame asked for in longer than the cache's patience is dropped, and its hold on the texture with it.
#[derive(Default)]
pub struct AttachmentViewCache {
  views: HashMap<(wgpu::Texture, u32, u32), (wgpu::TextureView, u64)>,
  frame: u64,
}

impl AttachmentViewCache {
  /// Frames an unused view is kept for before it is dropped.
  pub const MAX_IDLE_FRAMES: u64 = 30;

  /// Starts a frame: drops the views the frames before it left unused for too long.
  pub(crate) fn begin_frame(&mut self) {
    self.frame += 1;

    let frame: u64 = self.frame;

    self
      .views
      .retain(|_, (_, last_frame)| frame - *last_frame <= Self::MAX_IDLE_FRAMES);
  }

  /// The two-dimensional view of `texture`'s `mip_level` and `array_layer`, made the first time a frame asks for it.
  pub(crate) fn get_view(
    &mut self,
    texture: &wgpu::Texture,
    (mip_level, array_layer): (u32, u32),
    label: &str,
  ) -> wgpu::TextureView {
    let frame: u64 = self.frame;
    let (view, last_frame) = self
      .views
      .entry((texture.clone(), mip_level, array_layer))
      .or_insert_with(|| {
        (
          texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(label),
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: mip_level,
            mip_level_count: Some(1),
            base_array_layer: array_layer,
            array_layer_count: Some(1),
            ..Default::default()
          }),
          frame,
        )
      });

    *last_frame = frame;
    view.clone()
  }

  pub fn get_view_count(&self) -> usize {
    self.views.len()
  }
}
