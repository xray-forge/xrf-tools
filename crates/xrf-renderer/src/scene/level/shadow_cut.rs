use crate::pass::view_binding::ViewBinding;

/// One cull and draw of a shadow's map, or of a region of it: the visible lists and draw arguments its cull fills, and
/// the camera it is culled and drawn by.
pub struct ShadowCut {
  /// Made by the first cull that reserves them, as large as the scene's lists, and let go of while it rests.
  lists: Option<wgpu::Buffer>,
  pub args: wgpu::Buffer,
  pub view: ViewBinding,
}

impl ShadowCut {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, label: &'static str, args_size: u64) -> Self {
    Self {
      lists: None,
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
    }
  }

  /// Makes the lists hold at least `size` bytes for a cull this frame, doubling them where they are too small; a cull
  /// fills them whole, so nothing they held is kept.
  pub fn reserve_lists(&mut self, device: &wgpu::Device, size: u64) {
    let held: u64 = self.lists.as_ref().map_or(0, wgpu::Buffer::size);

    if size <= held {
      return;
    }

    self.lists = Some(device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("shadow cut lists"),
      size: size.max(held * 2).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT * 64),
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
      mapped_at_creation: false,
    }));
  }

  /// The lists a cull this frame reserved.
  ///
  /// # Panics
  ///
  /// When no cull reserved them since the cut last rested.
  pub fn get_lists(&self) -> &wgpu::Buffer {
    self
      .lists
      .as_ref()
      .expect("a shadow cut's lists are reserved before it is culled")
  }

  /// Lets go of the lists, made again by the next cull.
  pub fn release_lists(&mut self) {
    self.lists = None;
  }

  /// The lists' bytes, while it holds them.
  pub fn get_memory(&self) -> u64 {
    self.lists.as_ref().map_or(0, wgpu::Buffer::size)
  }
}
