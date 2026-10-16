use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::frame::view_targets::ViewTargets;
use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::view_binding::ViewBinding;

/// Every quality, in the order their search pipelines are held.
const QUALITIES: [RenderAmbientOcclusionQuality; 4] = [
  RenderAmbientOcclusionQuality::Low,
  RenderAmbientOcclusionQuality::Medium,
  RenderAmbientOcclusionQuality::High,
  RenderAmbientOcclusionQuality::Ultra,
];

/// Ambient occlusion at half the frame's size: searched into its first target, denoised across into the second and
/// down back into the first, which combine reads.
pub struct AmbientOcclusionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The search, one a quality in `QUALITIES` order.
  searches: Vec<wgpu::RenderPipeline>,
  /// The denoise across, then down.
  denoises: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl AmbientOcclusionPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = AmbientOcclusionParameters::create_layout(device);
    let (searches, denoises) = Self::create_pipelines(source, view_layout, &layout)?;

    Ok(Self {
      searches,
      denoises,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout) {
        Ok((searches, denoises)) => {
          self.searches = searches;
          self.denoises = denoises;
        }
        Err(error) => log::error!("Ambient occlusion rejected, occluding with the last one: {error}"),
      }
    }
  }

  /// Which group each stage reads (the other target than it draws into), and which target it draws into: the search,
  /// then the denoise across, then back.
  pub const STAGES: [(usize, usize); 3] = [(0, 0), (1, 1), (0, 0)];

  /// Draws one of its stages into the target the pass draws into: the search at `quality`, or a denoise.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (stage, quality): (usize, RenderAmbientOcclusionQuality),
    view: &ViewBinding,
    parameters: &AmbientOcclusionParameters,
  ) {
    let search: usize = QUALITIES.iter().position(|it| *it == quality).unwrap_or(2);
    let pipeline: &wgpu::RenderPipeline = match stage {
      0 => &self.searches[search],
      _ => &self.denoises[stage - 1],
    };

    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  #[allow(clippy::type_complexity)]
  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, [wgpu::RenderPipeline; 2])> {
    let device: &wgpu::Device = source.device;

    let module: wgpu::ShaderModule = source.get_module("frame/ambient_occlusion")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("ambient occlusion"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create = |fragment: &str, constants: &[(&str, f64)]| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "ambient occlusion", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("ambient occlusion"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: wgpu::PipelineCompilationOptions {
              constants,
              ..Default::default()
            },
            targets: &[Some(ViewTargets::OCCLUSION.into())],
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: source.get_driver_cache(),
        })
      })
    };
    let mut searches: Vec<wgpu::RenderPipeline> = Vec::with_capacity(QUALITIES.len());

    for quality in QUALITIES {
      let (slices, steps) = quality.get_search();

      searches.push(create(
        "fs_search",
        &[("SLICES", slices as f64), ("STEPS", steps as f64)],
      )?);
    }

    Ok((searches, [create("fs_denoise_x", &[])?, create("fs_denoise_y", &[])?]))
  }
}
