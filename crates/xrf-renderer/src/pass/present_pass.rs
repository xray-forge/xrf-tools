use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Puts a viewport's finished scene into its rectangle of the window, moved where the water and the particles distort it and dithered to
/// the window's eight bits; or, for a debug view, one of the targets the scene was built from.
pub struct PresentPass {
  layout: wgpu::BindGroupLayout,
  /// The bloom's, `smp_rtlinear`: filtered, clamped to the edge.
  bloom_sampler: wgpu::Sampler,
  view_layout: wgpu::BindGroupLayout,
  /// One a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  generation: u64,
}

impl PresentPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> Self {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("present"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, wgpu::TextureSampleType::Depth, flat),
        texture_entry(3, fragment, unfiltered, flat),
        texture_entry(4, fragment, unfiltered, flat),
        texture_entry(5, fragment, unfiltered, flat),
        texture_entry(6, fragment, unfiltered, flat),
        texture_entry(7, fragment, unfiltered, flat),
        uniform_entry(8, fragment),
        texture_entry(9, fragment, unfiltered, flat),
        texture_entry(10, fragment, unfiltered, flat),
        texture_entry(11, fragment, wgpu::TextureSampleType::Float { filterable: true }, flat),
        wgpu::BindGroupLayoutEntry {
          binding: 12,
          visibility: fragment,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    Self {
      layout,
      bloom_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("present bloom"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      pipelines: HashMap::new(),
      generation: shaders.get_generation(),
    }
  }

  /// Forgets pipelines built from a library reloaded since, so the next frame builds them again.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();
      self.pipelines.clear();
    }
  }

  /// Builds the pipeline drawing into a window format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = create_fullscreen_pipeline(
        device,
        shaders,
        "frame/present",
        "fs_present",
        &[Some(&self.view_layout), Some(&self.layout)],
        format,
      )?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  /// Binds the targets it shows, the uniform saying which, a `PresentUniform`, and the upscaled frame, or the scene
  /// again where it is drawn at the viewport's size.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    uniform: &wgpu::Buffer,
    upscaled: Option<&wgpu::TextureView>,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("present"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.scene),
        texture_binding(1, &targets.distortion),
        texture_binding(2, &targets.depth),
        texture_binding(3, &targets.albedo),
        texture_binding(4, &targets.normal),
        texture_binding(5, &targets.material),
        texture_binding(6, &targets.light),
        texture_binding(7, &targets.occlusion[0]),
        buffer_binding(8, uniform),
        texture_binding(9, upscaled.unwrap_or(&targets.scene)),
        texture_binding(10, &targets.motion),
        texture_binding(11, &targets.bloom[0]),
        wgpu::BindGroupEntry {
          binding: 12,
          resource: wgpu::BindingResource::Sampler(&self.bloom_sampler),
        },
      ],
    })
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already.
  pub fn draw(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    format: wgpu::TextureFormat,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
  ) {
    if let Some(pipeline) = self.pipelines.get(&format) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(0, &view.bind_group, &[]);
      pass.set_bind_group(1, bind_group, &[]);
      pass.draw(0..3, 0..1);
    }
  }
}
