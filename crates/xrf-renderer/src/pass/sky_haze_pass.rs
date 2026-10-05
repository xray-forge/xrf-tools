use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline};
use crate::pass::layout_entries::{storage_entry, uniform_entry};
use crate::shader::shader_library::ShaderLibrary;

/// Draws the sky as the frame shows it, clouds and all, blurred into a viewport's haze map, which the distance fades
/// into: a few thousand texels a frame, since the skies blend as the clock moves and the clouds drift.
pub struct SkyHazePass {
  layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SkyHazePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, sky_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("sky haze"),
      entries: &[uniform_entry(0, fragment), storage_entry(1, fragment, false)],
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, &layout, sky_layout)?,
      sky_layout: sky_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.layout, &self.sky_layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky haze rejected, blurring with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    lighting: &wgpu::Buffer,
    exposure: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("sky haze"),
      layout: &self.layout,
      entries: &[buffer_binding(0, lighting), buffer_binding(1, exposure)],
    })
  }

  pub fn record(&self, pass: &mut wgpu::RenderPass<'_>, bind_group: &wgpu::BindGroup, sky_group: &wgpu::BindGroup) {
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(1, bind_group, &[]);
    pass.set_bind_group(2, sky_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/sky_haze",
      "fs_sky_haze",
      &[None, Some(layout), Some(sky_layout)],
      ViewTargets::HAZE,
    )
  }
}
