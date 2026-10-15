use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::param::bind_group_key::BindGroupKey;
use crate::param::binding_key::BindingKey;
use crate::param::pass_binding::PassBinding;
use crate::param::pass_parameters::PassParameters;
use crate::param::pass_resources::PassResources;

/// The layouts of pass parameters, one per struct, and the bind groups made from them, kept across frames by the
/// identity of what they bind: the same resources get the same bind group back, and one unused for longer than the
/// cache's patience is dropped. Shared by the passes recording a frame, which find what they bind under a read lock.
pub struct BindGroupCache {
  layouts: RwLock<HashMap<&'static str, wgpu::BindGroupLayout>>,
  /// Each group with the last frame that bound it, kept atomic so a found group is marked under the read lock.
  groups: RwLock<HashMap<BindGroupKey, (wgpu::BindGroup, AtomicU64)>>,
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
      layouts: RwLock::new(HashMap::new()),
      groups: RwLock::new(HashMap::new()),
      frame: AtomicU64::new(0),
      max_idle_frames: Self::MAX_IDLE_FRAMES,
    }
  }

  pub fn with_max_idle_frames(mut self, frames: u64) -> Self {
    self.max_idle_frames = frames;
    self
  }

  pub fn get_bind_group_count(&self) -> usize {
    self.groups.read().expect("bind group cache lock").len()
  }

  /// The layout of `P`'s bind group, made once; a pipeline built for `P` takes this one, so its bind groups match.
  pub fn get_layout<P: PassParameters>(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    if let Some(layout) = self.layouts.read().expect("bind group cache lock").get(P::LAYOUT_KEY) {
      return layout.clone();
    }

    self
      .layouts
      .write()
      .expect("bind group cache lock")
      .entry(P::LAYOUT_KEY)
      .or_insert_with(|| P::create_layout(device))
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
          PassBinding::BufferRange { buffer, offset, size } => {
            BindingKey::Buffer((*buffer).clone(), Some((*offset, *size)))
          }
          PassBinding::TextureView(view) => BindingKey::TextureView((*view).clone()),
          PassBinding::TextureViewArray(views) => {
            BindingKey::TextureViewArray(views.iter().map(|view| (*view).clone()).collect())
          }
          PassBinding::Sampler(sampler) => BindingKey::Sampler((*sampler).clone()),
          PassBinding::AccelerationStructure(structure) => BindingKey::AccelerationStructure(structure.key),
        })
        .collect(),
    };
    let frame: u64 = self.frame.load(Ordering::Relaxed);

    if let Some((group, last_frame)) = self.groups.read().expect("bind group cache lock").get(&key) {
      last_frame.store(frame, Ordering::Relaxed);

      return group.clone();
    }

    let mut groups = self.groups.write().expect("bind group cache lock");
    let (group, last_frame) = groups.entry(key).or_insert_with(|| {
      let entries: Vec<wgpu::BindGroupEntry<'_>> = bindings
        .iter()
        .zip(P::BINDINGS)
        .map(|(binding, index)| wgpu::BindGroupEntry {
          binding: *index,
          resource: match binding {
            PassBinding::Buffer(buffer) => buffer.as_entire_binding(),
            PassBinding::BufferRange { buffer, offset, size } => wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer,
              offset: *offset,
              size: wgpu::BufferSize::new(*size),
            }),
            PassBinding::TextureView(view) => wgpu::BindingResource::TextureView(view),
            PassBinding::TextureViewArray(views) => wgpu::BindingResource::TextureViewArray(views),
            PassBinding::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
            PassBinding::AccelerationStructure(structure) => structure.tlas.as_binding(),
          },
        })
        .collect();

      (
        device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some(P::LAYOUT_KEY),
          layout: &layout,
          entries: &entries,
        }),
        AtomicU64::new(frame),
      )
    });

    *last_frame.get_mut() = frame;
    group.clone()
  }

  /// Starts a frame: drops the bind groups the frames before it left unused for too long.
  pub(crate) fn begin_frame(&self) {
    let frame: u64 = self.frame.fetch_add(1, Ordering::Relaxed) + 1;
    let patience: u64 = self.max_idle_frames;

    self
      .groups
      .write()
      .expect("bind group cache lock")
      .retain(|_, (_, last_frame)| frame - *last_frame.get_mut() <= patience);
  }
}
