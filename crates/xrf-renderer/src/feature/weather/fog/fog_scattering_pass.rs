use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::weather::fog::fog_scattering_parameters::FogScatteringParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// The enhanced fog's scattering: the finished scene blurred down to a quarter and back up to half its size, then
/// blended into the scene where the fog lies and the blur is brighter, into a copy the scene takes back.
pub struct FogScatteringPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  pipelines: PipelineSet<FogScatteringPipelines>,
  /// What a blur reads in place of a blur before it: one texel of nothing.
  empty: wgpu::TextureView,
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
  /// The scattering's index in [`Self::STAGES`], past the blurs.
  pub const SCATTERING: usize = 2;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(queue: &wgpu::Queue, source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = FogScatteringParameters::create_layout(device);
    let pipelines: FogScatteringPipelines = Self::create_pipelines(source, view_layout, &layout)?;
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
      pipelines: PipelineSet::new(source.cache, pipelines),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fog scattering"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      empty,
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipelines.refresh(source.cache, || {
      Self::create_pipelines(source, &self.view_layout, &self.layout)
    }) {
      log::error!("Fog scattering rejected, scattering with the last one: {error}");
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

  /// Draws the `stage` of [`Self::STAGES`] into the target the pass draws into: a blur, or the scattering.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &FogScatteringParameters<'_>,
    stage: usize,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(if stage == Self::SCATTERING {
      &self.pipelines.scatter
    } else {
      &self.pipelines.blur
    });
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<FogScatteringPipelines> {
    let layouts: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let create = |fragment: &'static str, format: wgpu::TextureFormat| {
      create_fullscreen_pipeline(source, "frame/fog_scattering", fragment, &layouts, format)
    };

    Ok(FogScatteringPipelines {
      blur: create("fs_scatter_blur", Self::BLURRED)?,
      scatter: create("fs_scattering", ViewTargets::SCENE)?,
    })
  }
}

/// The scattering's pipelines: the blurs, and the scattering over the scene.
struct FogScatteringPipelines {
  blur: wgpu::RenderPipeline,
  scatter: wgpu::RenderPipeline,
}
