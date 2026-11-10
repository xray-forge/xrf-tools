use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::param::bind_group_key::BindGroupKey;
use crate::param::pass_binding::PassBinding;
use crate::param::pass_parameters::PassParameters;
use crate::param::pass_resources::PassResources;

/// The layouts of pass parameters, one per struct, and the bind groups made from them, kept across frames by the
/// identity of what they bind: the same resources get the same bind group back, and one unused for longer than the
/// cache's patience is dropped. Shared by the passes recording a frame, which find what they bind under a read lock,
/// by a hash of the bindings as they are, so a group found takes nothing from them (a bindless array's every view).
pub struct BindGroupCache {
  layouts: RwLock<HashMap<&'static str, wgpu::BindGroupLayout>>,
  /// The groups by the hash of what they bind (`BindGroupKey::hash_of`), those sharing one together.
  groups: RwLock<HashMap<u64, Vec<CachedGroup>>>,
  frame: AtomicU64,
  max_idle_frames: u64,
}

/// A cached bind group, what it binds, and the last frame that bound it, kept atomic so a found group is marked under
/// the read lock.
struct CachedGroup {
  key: BindGroupKey,
  group: wgpu::BindGroup,
  last_frame: AtomicU64,
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
    self
      .groups
      .read()
      .expect("bind group cache lock")
      .values()
      .map(Vec::len)
      .sum()
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
    let hash: u64 = BindGroupKey::hash_of(&layout, &bindings);
    let frame: u64 = self.frame.load(Ordering::Relaxed);

    if let Some(cached) = Self::find(
      self.groups.read().expect("bind group cache lock").get(&hash),
      &layout,
      &bindings,
    ) {
      cached.last_frame.store(frame, Ordering::Relaxed);

      return cached.group.clone();
    }

    let mut groups = self.groups.write().expect("bind group cache lock");
    let bucket: &mut Vec<CachedGroup> = groups.entry(hash).or_default();

    // Another recorder may have made it between the locks.
    if let Some(cached) = bucket.iter_mut().find(|cached| cached.key.is_of(&layout, &bindings)) {
      *cached.last_frame.get_mut() = frame;

      return cached.group.clone();
    }

    let group: wgpu::BindGroup = {
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

      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(P::LAYOUT_KEY),
        layout: &layout,
        entries: &entries,
      })
    };

    bucket.push(CachedGroup {
      key: BindGroupKey::new(&layout, &bindings),
      group: group.clone(),
      last_frame: AtomicU64::new(frame),
    });

    group
  }

  /// The group of a bucket that binds `bindings` by `layout`, if one does.
  fn find<'a>(
    bucket: Option<&'a Vec<CachedGroup>>,
    layout: &wgpu::BindGroupLayout,
    bindings: &[PassBinding<'_>],
  ) -> Option<&'a CachedGroup> {
    bucket?.iter().find(|cached| cached.key.is_of(layout, bindings))
  }

  /// Drops every bind group, and its hold on what it binds, keeping the layouts, for a renderer drawing no frames.
  pub(crate) fn clear_groups(&self) {
    self.groups.write().expect("bind group cache lock").clear();
  }

  /// Drops the bind groups the current frame did not bind, and their hold on what they bind, keeping the layouts.
  pub(crate) fn forget_unused(&self) {
    let frame: u64 = self.frame.load(Ordering::Relaxed);

    self.retain(|last_frame| last_frame == frame);
  }

  /// Keeps the groups whose last frame `is_kept` passes, and the buckets still holding one.
  fn retain(&self, is_kept: impl Fn(u64) -> bool) {
    self.groups.write().expect("bind group cache lock").retain(|_, bucket| {
      bucket.retain_mut(|cached| is_kept(*cached.last_frame.get_mut()));

      !bucket.is_empty()
    });
  }

  /// Starts a frame: drops the bind groups the frames before it left unused for too long.
  pub(crate) fn begin_frame(&self) {
    let frame: u64 = self.frame.fetch_add(1, Ordering::Relaxed) + 1;
    let patience: u64 = self.max_idle_frames;

    self.retain(|last_frame| frame - last_frame <= patience);
  }
}
