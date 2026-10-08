use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::reflection_history::ReflectionHistory;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Screen-space reflections at the size their quality traces at: each reflecting pixel's ray marched over the frame's
/// depth and what it met lit, blended with the last frame's into the view's history, then blurred twice into what
/// combine reads.
pub struct ReflectionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipelines: ReflectionPipelines,
  /// What a stage reads in place of a history or a stage before it where there is none: one texel of nothing.
  empty: wgpu::TextureView,
  /// What combine and the debug view read where nothing is traced: one texel marked untraced.
  untraced: wgpu::TextureView,
  generation: u64,
}

/// The trace, the blend over frames and the two blurs.
struct ReflectionPipelines {
  trace: wgpu::RenderPipeline,
  accumulate: wgpu::RenderPipeline,
  blur: wgpu::RenderPipeline,
  fine_blur: wgpu::RenderPipeline,
}

impl ReflectionPass {
  /// What the trace writes and the blurs leave: what is reflected, then one, below none where only the sky is.
  pub const TRACED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// How far along the view each ray's hit lies, in metres, nothing where it met none.
  pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = ReflectionParameters::create_layout(device);
    // Half floats: nothing, and nothing with an alpha of minus one.
    let texel = |label: &str, alpha: u16| -> wgpu::TextureView {
      device
        .create_texture_with_data(
          queue,
          &wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
              width: 1,
              height: 1,
              depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TRACED,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          },
          Default::default(),
          bytemuck::cast_slice(&[0u16, 0, 0, alpha]),
        )
        .create_view(&Default::default())
    };

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, (view_layout, &layout, sky_layout))?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      empty: texel("reflections empty", 0),
      untraced: texel("reflections none", 0xbc00),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, (&self.view_layout, &self.layout, &self.sky_layout)) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Reflections rejected, tracing with the last ones: {error}"),
      }
    }
  }

  /// The texel a stage reads where no history or stage before it is bound.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// The texel combine and the debug view read where nothing is traced.
  pub fn get_untraced(&self) -> &wgpu::TextureView {
    &self.untraced
  }

  /// Traces every reflecting pixel's ray into the targets the pass draws into.
  pub fn record_trace(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
    sky: &SkyParameters<'_>,
  ) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.trace);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  /// Blends the trace with the last frame's reflections into this frame's.
  pub fn record_accumulate(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
  ) {
    Self::draw(context, &self.pipelines.accumulate, view, parameters);
  }

  /// Blurs the reflections, its taps a traced pixel apart, or a frame's pixel apart where `is_fine`.
  pub fn record_blur(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
    is_fine: bool,
  ) {
    let pipeline: &wgpu::RenderPipeline = if is_fine {
      &self.pipelines.fine_blur
    } else {
      &self.pipelines.blur
    };

    Self::draw(context, pipeline, view, parameters);
  }

  fn draw(
    context: &mut RasterContext<'_>,
    pipeline: &wgpu::RenderPipeline,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    (view_layout, layout, sky_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<ReflectionPipelines> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/reflections")?;
    let traced_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("reflection trace"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(sky_layout)],
      ..Default::default()
    });
    let stage_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("reflections"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create = |pipeline_layout: &wgpu::PipelineLayout,
                  fragment: &str,
                  formats: &[wgpu::TextureFormat]|
     -> XrfResult<wgpu::RenderPipeline> {
      let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|it| Some((*it).into())).collect();

      create_checked(device, fragment, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some(fragment),
          layout: Some(pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets: &targets,
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    Ok(ReflectionPipelines {
      trace: create(&traced_layout, "fs_trace", &[Self::TRACED, Self::DEPTH])?,
      accumulate: create(
        &stage_layout,
        "fs_accumulate",
        &[ReflectionHistory::COLOUR_FORMAT, ReflectionHistory::HELD_FORMAT],
      )?,
      blur: create(&stage_layout, "fs_blur", &[Self::TRACED])?,
      fine_blur: create(&stage_layout, "fs_blur_fine", &[Self::TRACED])?,
    })
  }
}
