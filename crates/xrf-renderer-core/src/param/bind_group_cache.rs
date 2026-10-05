use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::param::bind_group_key::BindGroupKey;
use crate::param::binding_key::BindingKey;
use crate::param::pass_binding::PassBinding;
use crate::param::pass_parameters::PassParameters;
use crate::param::pass_resources::PassResources;

/// The layouts of pass parameters, one per struct, and the bind groups made from them, kept across frames by the
/// identity of what they bind: the same resources get the same bind group back, and one unused for longer than the
/// cache's patience is dropped. Shared by the passes recording a frame, so it locks.
pub struct BindGroupCache {
  layouts: Mutex<HashMap<&'static str, wgpu::BindGroupLayout>>,
  groups: Mutex<HashMap<BindGroupKey, (wgpu::BindGroup, u64)>>,
  frame: AtomicU64,
  max_idle_frames: u64,
}

impl Default for BindGroupCache {
  fn default() -> Self {
    Self::new()
  }
}

impl BindGroupCache {
  /// Frames an unused bind group is kept for before it is dropped.
  pub const MAX_IDLE_FRAMES: u64 = 30;

  pub fn new() -> Self {
    Self {
      layouts: Mutex::new(HashMap::new()),
      groups: Mutex::new(HashMap::new()),
      frame: AtomicU64::new(0),
      max_idle_frames: Self::MAX_IDLE_FRAMES,
    }
  }

  pub fn with_max_idle_frames(mut self, frames: u64) -> Self {
    self.max_idle_frames = frames;
    self
  }

  pub fn get_bind_group_count(&self) -> usize {
    self.groups.lock().expect("bind group cache lock").len()
  }

  /// The layout of `P`'s bind group, made once; a pipeline built for `P` takes this one, so its bind groups match.
  pub fn get_layout<P: PassParameters>(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    self
      .layouts
      .lock()
      .expect("bind group cache lock")
      .entry(P::LAYOUT_KEY)
      .or_insert_with(|| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
          label: Some(P::LAYOUT_KEY),
          entries: &P::get_layout_entries(),
        })
      })
      .clone()
  }

  /// The bind group binding what `parameters` name this frame: the cached one where it binds the same resources.
  pub fn get_bind_group<'r, P: PassParameters>(
    &self,
    device: &wgpu::Device,
    parameters: &'r P,
    resources: &dyn PassResources<'r>,
  ) -> wgpu::BindGroup {
    let layout: wgpu::BindGroupLayout = self.get_layout::<P>(device);
    let bindings: Vec<PassBinding<'r>> = parameters.list_bindings(resources);
    let key: BindGroupKey = BindGroupKey {
      layout: layout.clone(),
      bindings: bindings
        .iter()
        .map(|binding| match binding {
          PassBinding::Buffer(buffer) => BindingKey::Buffer((*buffer).clone(), None),
          PassBinding::BufferRange { buffer, size } => BindingKey::Buffer((*buffer).clone(), Some(*size)),
          PassBinding::TextureView(view) => BindingKey::TextureView((*view).clone()),
          PassBinding::Sampler(sampler) => BindingKey::Sampler((*sampler).clone()),
        })
        .collect(),
    };
    let frame: u64 = self.frame.load(Ordering::Relaxed);
    let mut groups = self.groups.lock().expect("bind group cache lock");
    let (group, last_frame) = groups.entry(key).or_insert_with(|| {
      let entries: Vec<wgpu::BindGroupEntry<'_>> = bindings
        .iter()
        .enumerate()
        .map(|(index, binding)| wgpu::BindGroupEntry {
          binding: index as u32,
          resource: match binding {
            PassBinding::Buffer(buffer) => buffer.as_entire_binding(),
            PassBinding::BufferRange { buffer, size } => wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer,
              offset: 0,
              size: wgpu::BufferSize::new(*size),
            }),
            PassBinding::TextureView(view) => wgpu::BindingResource::TextureView(view),
            PassBinding::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
          },
        })
        .collect();

      (
        device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some(P::LAYOUT_KEY),
          layout: &layout,
          entries: &entries,
        }),
        frame,
      )
    });

    *last_frame = frame;
    group.clone()
  }

  /// Starts a frame: drops the bind groups the frames before it left unused for too long.
  pub(crate) fn begin_frame(&self) {
    let frame: u64 = self.frame.fetch_add(1, Ordering::Relaxed) + 1;
    let patience: u64 = self.max_idle_frames;

    self
      .groups
      .lock()
      .expect("bind group cache lock")
      .retain(|_, (_, last_frame)| frame - *last_frame <= patience);
  }
}
