use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters, RasterContext};

use crate::frame::reflection_depth::ReflectionDepth;
use crate::frame::reflection_history::ReflectionHistory;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a reduction workgroup runs a side, as `shaders/frame/reflection_depth.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Screen-space reflections at the size their quality traces at: the depth reduced to its nearest, each glossy pixel's
/// ray marched over it and what it met lit, accumulated over frames into the view's history, then filtered into what
/// combine reads.
pub struct ReflectionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  depth_layout: wgpu::BindGroupLayout,
  pipelines: ReflectionPipelines,
  /// What a stage reads in place of a history or a stage before it where there is none: one texel of nothing.
  empty: wgpu::TextureView,
  /// What combine and the debug view read where nothing is traced: one texel marked untraced.
  untraced: wgpu::TextureView,
  generation: u64,
}

/// The depth's reduction, the trace, the accumulation and the filter.
struct ReflectionPipelines {
  depth: wgpu::ComputePipeline,
  trace: wgpu::RenderPipeline,
  accumulate: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
}

impl ReflectionPass {
  /// What the trace writes and the filter leaves: radiance met times trust, then trust, below none where untraced.
  pub const TRACED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// How far each ray went, in metres.
  pub const LENGTH: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

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
    let depth_layout: wgpu::BindGroupLayout = PyramidDepthParameters::create_layout(device);
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
      pipelines: Self::create_pipelines(device, shaders, (view_layout, &layout, sky_layout), &depth_layout)?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      empty: texel("reflections empty", 0),
      untraced: texel("reflections none", 0xbc00),
      generation: shaders.get_generation(),
      layout,
      depth_layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(
        device,
        shaders,
        (&self.view_layout, &self.layout, &self.sky_layout),
        &self.depth_layout,
      ) {
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

  /// Reduces the depth to its nearest at the size traced.
  pub fn record_depth(
    &self,
    context: &mut ComputeContext<'_>,
    depth: &ReflectionDepth,
    parameters: &PyramidDepthParameters,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.depth);
    pass.dispatch_workgroups(depth.width.div_ceil(WORKGROUP), depth.height.div_ceil(WORKGROUP), 1);
  }

  /// Traces every glossy pixel's ray into the targets the pass draws into.
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

  /// Blends the trace with the last frame's accumulation into this frame's.
  pub fn record_accumulate(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
  ) {
    Self::draw(context, &self.pipelines.accumulate, view, parameters);
  }

  /// Filters the accumulation, or the trace alone, into what combine reads.
  pub fn record_filter(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &ReflectionParameters<'_>,
  ) {
    Self::draw(context, &self.pipelines.filter, view, parameters);
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
    depth_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<ReflectionPipelines> {
    let depth_module: wgpu::ShaderModule = create_module(device, shaders, "frame/reflection_depth")?;
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/reflections")?;
    let reduce = |entry: &str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
      let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(entry),
        bind_group_layouts: &[Some(layout)],
        ..Default::default()
      });

      create_checked(device, entry, || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some(entry),
          layout: Some(&pipeline_layout),
          module: &depth_module,
          entry_point: Some(entry),
          compilation_options: Default::default(),
          cache: None,
        })
      })
    };
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
      depth: reduce("reduce_nearest_depth", depth_layout)?,
      trace: create(&traced_layout, "fs_trace", &[Self::TRACED, Self::LENGTH])?,
      accumulate: create(
        &stage_layout,
        "fs_accumulate",
        &[ReflectionHistory::COLOUR_FORMAT, ReflectionHistory::HELD_FORMAT],
      )?,
      filter: create(&stage_layout, "fs_filter", &[Self::TRACED])?,
    })
  }
}
