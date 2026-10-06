use std::collections::HashMap;
use std::sync::Mutex;

use xrf_error::{XrfError, XrfResult};

use crate::pipeline::compute_pipeline_description::ComputePipelineDescription;
use crate::pipeline::render_pipeline_description::RenderPipelineDescription;
use crate::pipeline::shader_source::ShaderSource;

/// The shader modules and pipelines a renderer has made, kept by what they were made from: a module by name, a
/// pipeline by its whole description, permutation and bind group layouts included. Asked for one it has, it hands it
/// back; asked for one it lacks, it makes it, so precompiling a level's pipelines is asking for them ahead. A module
/// hot reload changed is invalidated with every pipeline made from it.
#[derive(Default)]
pub struct PipelineCache {
  modules: Mutex<HashMap<&'static str, wgpu::ShaderModule>>,
  render: Mutex<HashMap<RenderPipelineDescription, wgpu::RenderPipeline>>,
  compute: Mutex<HashMap<ComputePipelineDescription, wgpu::ComputePipeline>>,
}

impl PipelineCache {
  pub fn new() -> Self {
    Self::default()
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
    let layout: wgpu::PipelineLayout = Self::create_layout(device, description.label, &description.bind_group_layouts);
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
        cache: None,
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
    let layout: wgpu::PipelineLayout = Self::create_layout(device, description.label, &description.bind_group_layouts);
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
        cache: None,
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
  }

  /// Forgets every module and pipeline.
  pub fn invalidate_all(&self) {
    self.modules.lock().expect("pipeline cache lock").clear();
    self.render.lock().expect("pipeline cache lock").clear();
    self.compute.lock().expect("pipeline cache lock").clear();
  }

  fn get_module(
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
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(module),
        source: wgpu::ShaderSource::Wgsl(code.into()),
      })
    })?;

    self
      .modules
      .lock()
      .expect("pipeline cache lock")
      .insert(module, compiled.clone());

    Ok(compiled)
  }

  fn create_layout(device: &wgpu::Device, label: &str, layouts: &[wgpu::BindGroupLayout]) -> wgpu::PipelineLayout {
    let layouts: Vec<Option<&wgpu::BindGroupLayout>> = layouts.iter().map(Some).collect();

    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(label),
      bind_group_layouts: &layouts,
      ..Default::default()
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
