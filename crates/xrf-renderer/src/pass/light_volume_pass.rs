use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::light_volume_parameters::LightVolumeParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Draws the shadowed local lights' shafts (`accum_volumetric`) into a target of their own at half the frame's size,
/// slices added where the scene stands behind them, tested against its depth taken at that size without writing it,
/// for the shafts' composite to bring up and add to the frame with the sun's.
pub struct LightVolumePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  /// The slices, then the half-size depth they are tested against.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl LightVolumePass {
  /// What the shafts add up in, as `rt_Generic_2` holds them.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = LightVolumeParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, [view_layout, &layout, texture_layout])?,
      layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: source.get_generation(),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, [&self.view_layout, &self.layout, &self.texture_layout]) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Light volumes rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Draws each volume's slices, `slices` a volume in the parameters' order.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &LightVolumeParameters,
    textures: &wgpu::BindGroup,
    slices: &[u32],
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[0]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);

    for (volume, count) in slices.iter().enumerate() {
      let volume: u32 = volume as u32;

      pass.draw(0..count * 6, volume..volume + 1);
    }
  }

  /// Takes the frame's depth at half its size into the depth target the slices are tested against.
  pub fn record_depth(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &LightVolumeParameters,
    textures: &wgpu::BindGroup,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[1]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    layouts: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let bind_group_layouts: [Option<&wgpu::BindGroupLayout>; 3] = layouts.map(Some);
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };

    let slices: wgpu::RenderPipeline = source.get_render_pipeline(
      &RenderPipelineDescription::new("light volumes", "frame/light_volumes", "vs_light_volume")
        .with_fragment("fs_light_volume")
        .with_bind_group_layouts(&bind_group_layouts)
        // `accum_volumetric.s`: both sides, the scene's depth tested and never written.
        .with_primitive(wgpu::PrimitiveState {
          cull_mode: None,
          ..Default::default()
        })
        .with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(false),
          depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
          stencil: Default::default(),
          bias: Default::default(),
        })
        .with_target(wgpu::ColorTargetState {
          format: Self::FORMAT,
          blend: Some(wgpu::BlendState {
            color: added,
            alpha: added,
          }),
          write_mask: wgpu::ColorWrites::COLOR,
        }),
    )?;
    let depth: wgpu::RenderPipeline = source.get_render_pipeline(
      &RenderPipelineDescription::new("light volumes depth", "frame/light_volumes", "vs_fullscreen")
        .with_fragment("fs_volume_depth")
        .with_bind_group_layouts(&bind_group_layouts)
        .with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Always),
          stencil: Default::default(),
          bias: Default::default(),
        }),
    )?;

    Ok([slices, depth])
  }
}
