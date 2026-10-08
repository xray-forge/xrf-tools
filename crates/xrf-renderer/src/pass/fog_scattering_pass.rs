use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fog_scattering_parameters::FogScatteringParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// The enhanced fog's scattering: the finished scene blurred down to a quarter and back up to half its size, then
/// blended into the scene where the fog lies and the blur is brighter, into a copy the scene takes back.
pub struct FogScatteringPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  blur: wgpu::RenderPipeline,
  scatter: wgpu::RenderPipeline,
  /// What a blur reads in place of a blur before it: one texel of nothing.
  empty: wgpu::TextureView,
  generation: u64,
}

impl FogScatteringPass {
  /// The blurs' targets: eight bits a channel, as the engine's blur targets are.
  pub const BLURRED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  /// Its stages: the blur to a quarter of the frame, the blur to half of it, the scattering, and the copy back.
  pub const STAGES: [&'static str; 4] = [
    "fog scatter quarter",
    "fog scatter half",
    "fog scattering",
    "fog scattering copy",
  ];

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = FogScatteringParameters::create_layout(device);
    let (blur, scatter) = Self::create_pipelines(device, shaders, (view_layout, &layout))?;
    let empty: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("fog scattering empty"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::BLURRED,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &[0, 0, 0, 0],
      )
      .create_view(&Default::default());

    Ok(Self {
      blur,
      scatter,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fog scattering"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      empty,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, (&self.view_layout, &self.layout)) {
        Ok((blur, scatter)) => (self.blur, self.scatter) = (blur, scatter),
        Err(error) => log::error!("Fog scattering rejected, scattering with the last one: {error}"),
      }
    }
  }

  /// The sampler its stages read with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// The texel a blur reads where no blur before it is bound.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// Draws a blur, or the scattering where `is_scattering`, into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &FogScatteringParameters<'_>,
    is_scattering: bool,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(if is_scattering { &self.scatter } else { &self.blur });
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    (view_layout, layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<(wgpu::RenderPipeline, wgpu::RenderPipeline)> {
    let layouts: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let create = |fragment: &str, format: wgpu::TextureFormat| {
      create_fullscreen_pipeline(device, shaders, "frame/fog_scattering", fragment, &layouts, format)
    };

    Ok((
      create("fs_scatter_blur", Self::BLURRED)?,
      create("fs_scattering", ViewTargets::SCENE)?,
    ))
  }
}
