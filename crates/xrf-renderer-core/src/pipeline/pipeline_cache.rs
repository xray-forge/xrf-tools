use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_error::{XrfError, XrfResult};

use crate::pipeline::compute_pipeline_description::ComputePipelineDescription;
use crate::pipeline::render_pipeline_description::RenderPipelineDescription;
use crate::pipeline::shader_source::ShaderSource;

/// The shader modules and pipelines a renderer has made, kept by what they were made from: a module by name, a
/// pipeline by its whole description, permutation and bind group layouts included. Asked for one it has, it hands it
/// back; asked for one it lacks, it makes it, so precompiling a level's pipelines is asking for them ahead. A module
/// hot reload changed is invalidated with every pipeline made from it. Every pipeline is made with the driver's own
/// pipeline cache where the device keeps one, which a renderer saves and hands back at its next start.
#[derive(Default)]
pub struct PipelineCache {
  modules: Mutex<HashMap<&'static str, wgpu::ShaderModule>>,
  render: Mutex<HashMap<RenderPipelineDescription, wgpu::RenderPipeline>>,
  compute: Mutex<HashMap<ComputePipelineDescription, wgpu::ComputePipeline>>,
  /// Bumped by every invalidation, so whoever keeps a pipeline it was handed knows to ask again.
  generation: AtomicU64,
  /// The driver's cache of compiled pipelines, where the device keeps one.
  driver: Option<wgpu::PipelineCache>,
}

impl PipelineCache {
  /// The checks compiled into every module: none on indexing or loop counts, which the shaders and the hardware's own
  /// buffer bounds make redundant and which cost every buffer read and loop, the guards on division and ray queries
  /// kept.
  const RUNTIME_CHECKS: wgpu::ShaderRuntimeChecks = wgpu::ShaderRuntimeChecks {
    bounds_checks: false,
    force_loop_bounding: false,
    ..wgpu::ShaderRuntimeChecks::checked()
  };

  pub fn new() -> Self {
    Self::default()
  }

  /// A cache whose pipelines are made with a driver cache where `device` keeps one, started from `data` that
  /// `get_driver_data` answered for the same adapter and driver before; data whose checksum does not hold starts it
  /// empty.
  pub fn with_driver_cache(device: &wgpu::Device, data: Option<&[u8]>) -> Self {
    let data: Option<&[u8]> = data.and_then(|data| {
      let checked: Option<&[u8]> = Self::read_checked(data);

      if checked.is_none() {
        log::warn!("The driver's kept pipelines do not match their checksum, so every pipeline compiles again");
      }

      checked
    });
    let driver: Option<wgpu::PipelineCache> = device.features().contains(wgpu::Features::PIPELINE_CACHE).then(|| {
      // SAFETY: the data is what `get_driver_data` answered, its checksum holding, so it is intact; wgpu refuses data of
      // another adapter, driver or size, starting the cache empty in its place.
      unsafe {
        device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
          label: Some("pipelines"),
          data,
          fallback: true,
        })
      }
    });

    Self {
      driver,
      ..Self::default()
    }
  }

  /// What the driver's cache holds behind its checksum, to hand back at the next start; none where the device keeps
  /// none.
  pub fn get_driver_data(&self) -> Option<Vec<u8>> {
    self
      .driver
      .as_ref()
      .and_then(wgpu::PipelineCache::get_data)
      .map(|data| Self::to_checked(&data))
  }

  /// The driver's data behind its checksum.
  pub(crate) fn to_checked(data: &[u8]) -> Vec<u8> {
    let mut checked: Vec<u8> = Vec::with_capacity(data.len() + 4);

    checked.extend_from_slice(&crc32fast::hash(data).to_le_bytes());
    checked.extend_from_slice(data);

    checked
  }

  /// The driver's data behind a checksum `to_checked` wrote, where the checksum holds.
  pub(crate) fn read_checked(data: &[u8]) -> Option<&[u8]> {
    let (checksum, data): (&[u8; 4], &[u8]) = data.split_first_chunk::<4>()?;

    (u32::from_le_bytes(*checksum) == crc32fast::hash(data)).then_some(data)
  }

  /// How many invalidations the cache has had: a pipeline handed out before the latest may be stale.
  pub fn get_generation(&self) -> u64 {
    self.generation.load(Ordering::Acquire)
  }

  /// Pipelines held, render and compute.
  pub fn get_pipeline_count(&self) -> usize {
    self.render.lock().expect("pipeline cache lock").len() + self.compute.lock().expect("pipeline cache lock").len()
  }

  /// The render pipeline `description` describes, made once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses the module or the pipeline.
  pub fn get_render_pipeline(
    &self,
    device: &wgpu::Device,
    source: &dyn ShaderSource,
    description: &RenderPipelineDescription,
  ) -> XrfResult<wgpu::RenderPipeline> {
    if let Some(pipeline) = self.render.lock().expect("pipeline cache lock").get(description) {
      return Ok(pipeline.clone());
    }

    let module: wgpu::ShaderModule = self.get_module(device, source, description.module)?;
    let layout: wgpu::PipelineLayout = Self::create_layout(
      device,
      description.label,
      (&description.bind_group_layouts, description.immediate_size),
    );
    let constants: Vec<(&'static str, f64)> = description.constants.to_wgpu();
    let compilation_options = wgpu::PipelineCompilationOptions {
      constants: &constants,
      ..Default::default()
    };
    let vertex_layouts: Vec<Option<wgpu::VertexBufferLayout<'_>>> = description
      .vertex_layouts
      .iter()
      .map(|layout| Some(layout.to_wgpu()))
      .collect();
    let pipeline: wgpu::RenderPipeline = Self::create_checked(device, description.label, || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(description.label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some(description.vertex_entry),
          compilation_options: compilation_options.clone(),
          buffers: &vertex_layouts,
        },
        primitive: description.primitive,
        depth_stencil: description.depth_stencil.clone(),
        multisample: description.multisample,
        fragment: description.fragment_entry.map(|entry| wgpu::FragmentState {
          module: &module,
          entry_point: Some(entry),
          compilation_options: compilation_options.clone(),
          targets: &description.targets,
        }),
        multiview_mask: None,
        cache: self.driver.as_ref(),
      })
    })?;

    self
      .render
      .lock()
      .expect("pipeline cache lock")
      .insert(description.clone(), pipeline.clone());

    Ok(pipeline)
  }

  /// The compute pipeline `description` describes, made once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses the module or the pipeline.
  pub fn get_compute_pipeline(
    &self,
    device: &wgpu::Device,
    source: &dyn ShaderSource,
    description: &ComputePipelineDescription,
  ) -> XrfResult<wgpu::ComputePipeline> {
    if let Some(pipeline) = self.compute.lock().expect("pipeline cache lock").get(description) {
      return Ok(pipeline.clone());
    }

    let module: wgpu::ShaderModule = self.get_module(device, source, description.module)?;
    let layout: wgpu::PipelineLayout =
      Self::create_layout(device, description.label, (&description.bind_group_layouts, 0));
    let constants: Vec<(&'static str, f64)> = description.constants.to_wgpu();
    let pipeline: wgpu::ComputePipeline = Self::create_checked(device, description.label, || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(description.label),
        layout: Some(&layout),
        module: &module,
        entry_point: Some(description.entry_point),
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &constants,
          ..Default::default()
        },
        cache: self.driver.as_ref(),
      })
    })?;

    self
      .compute
      .lock()
      .expect("pipeline cache lock")
      .insert(description.clone(), pipeline.clone());

    Ok(pipeline)
  }

  /// Makes every pipeline described ahead of the frames that draw with them, so none is compiled mid-frame.
  ///
  /// # Errors
  ///
  /// Returns the first pipeline's error; those before it stay made.
  pub fn warm(
    &self,
    device: &wgpu::Device,
    source: &dyn ShaderSource,
    render: &[RenderPipelineDescription],
    compute: &[ComputePipelineDescription],
  ) -> XrfResult {
    for description in render {
      self.get_render_pipeline(device, source, description)?;
    }

    for description in compute {
      self.get_compute_pipeline(device, source, description)?;
    }

    Ok(())
  }

  /// Forgets a module and every pipeline made from it, so the next ask composes it afresh.
  pub fn invalidate_module(&self, module: &str) {
    self.modules.lock().expect("pipeline cache lock").remove(module);
    self
      .render
      .lock()
      .expect("pipeline cache lock")
      .retain(|description, _| description.module != module);
    self
      .compute
      .lock()
      .expect("pipeline cache lock")
      .retain(|description, _| description.module != module);
    self.generation.fetch_add(1, Ordering::AcqRel);
  }

  /// Forgets every module and pipeline.
  pub fn invalidate_all(&self) {
    self.modules.lock().expect("pipeline cache lock").clear();
    self.render.lock().expect("pipeline cache lock").clear();
    self.compute.lock().expect("pipeline cache lock").clear();
    self.generation.fetch_add(1, Ordering::AcqRel);
  }

  /// The module `module` names, composed and compiled once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses it.
  pub fn get_module(
    &self,
    device: &wgpu::Device,
    source: &dyn ShaderSource,
    module: &'static str,
  ) -> XrfResult<wgpu::ShaderModule> {
    if let Some(compiled) = self.modules.lock().expect("pipeline cache lock").get(module) {
      return Ok(compiled.clone());
    }

    let code: String = source.compose(module)?;
    let compiled: wgpu::ShaderModule = Self::create_checked(device, module, || {
      // SAFETY: every module is the renderer's own. Its loops each end (a counted bound, a float bound a NaN also
      // ends, or a ray query's traversal), and its storage buffers are bound through descriptors, which both backends
      // bound-check in hardware (D3D12's views, Vulkan's robust buffer access), as they do image loads; the dynamic
      // uniforms are read at fixed fields and constant-bounded indices. A texture binding array, which no hardware
      // checks and which is bound only up to the slots handed out, is indexed only by slots the renderer itself wrote
      // into its surface, model and light records from what its texture cache handed out; level data reaches it only
      // as an index into those records, a storage buffer. Workgroup arrays are indexed by the invocation's own index
      // within a workgroup their size, and function-local arrays by vertex and lane counts the renderer draws and
      // dispatches. Division and ray queries stay guarded.
      unsafe {
        device.create_shader_module_trusted(
          wgpu::ShaderModuleDescriptor {
            label: Some(module),
            source: wgpu::ShaderSource::Wgsl(code.into()),
          },
          Self::RUNTIME_CHECKS,
        )
      }
    })?;

    self
      .modules
      .lock()
      .expect("pipeline cache lock")
      .insert(module, compiled.clone());

    Ok(compiled)
  }

  fn create_layout(
    device: &wgpu::Device,
    label: &str,
    (layouts, immediate_size): (&[Option<wgpu::BindGroupLayout>], u32),
  ) -> wgpu::PipelineLayout {
    let layouts: Vec<Option<&wgpu::BindGroupLayout>> = layouts.iter().map(Option::as_ref).collect();

    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(label),
      bind_group_layouts: &layouts,
      immediate_size,
    })
  }

  /// Runs a creation under a validation scope, so what wgpu refuses is an error rather than a later failure.
  fn create_checked<T>(device: &wgpu::Device, label: &str, create: impl FnOnce() -> T) -> XrfResult<T> {
    let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let created: T = create();

    match pollster::block_on(scope.pop()) {
      None => Ok(created),
      Some(error) => Err(XrfError::new_invalid_error(format!("'{label}': {error}"))),
    }
  }
}
