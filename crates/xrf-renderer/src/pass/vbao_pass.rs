use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::frame::vbao_history::VbaoHistory;
use crate::frame::view_targets::ViewTargets;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Every quality, in the order their search pipelines are held.
const QUALITIES: [RenderAmbientOcclusionQuality; 4] = [
  RenderAmbientOcclusionQuality::Low,
  RenderAmbientOcclusionQuality::Medium,
  RenderAmbientOcclusionQuality::High,
  RenderAmbientOcclusionQuality::Ultra,
];

/// VBAO, the visibility-bitmask ambient occlusion, at half the frame's size: searched into the first occlusion
/// target, accumulated over frames into the view's history, then filtered into the second, which combine reads.
pub struct VbaoPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipelines: VbaoPipelines,
  /// What a stage reads in place of a history where there is none: one texel, all visible and nothing drawn.
  empty: wgpu::TextureView,
  generation: u64,
}

/// The search a quality, the accumulation and the filter.
struct VbaoPipelines {
  searches: Vec<wgpu::RenderPipeline>,
  accumulate: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
}

impl VbaoPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = VbaoParameters::create_layout(device);
    let empty: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("vbao history empty"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &[u8::MAX, 0, 0, 0],
      )
      .create_view(&Default::default());

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      empty,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("VBAO rejected, occluding with the last one: {error}"),
      }
    }
  }

  /// The texel a stage reads where no history is bound.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// Searches the frame's pixels at `quality` into the target the pass draws into.
  pub fn record_search(
    &self,
    context: &mut RasterContext<'_>,
    quality: RenderAmbientOcclusionQuality,
    view: &ViewBinding,
    parameters: &VbaoParameters,
  ) {
    let search: usize = QUALITIES.iter().position(|it| *it == quality).unwrap_or(2);

    Self::draw(context, &self.pipelines.searches[search], view, parameters);
  }

  /// Blends the search with the last frame's accumulation into this frame's.
  pub fn record_accumulate(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &VbaoParameters) {
    Self::draw(context, &self.pipelines.accumulate, view, parameters);
  }

  /// Filters the accumulation, or the search alone, into the occlusion combine reads.
  pub fn record_filter(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &VbaoParameters) {
    Self::draw(context, &self.pipelines.filter, view, parameters);
  }

  fn draw(
    context: &mut RasterContext<'_>,
    pipeline: &wgpu::RenderPipeline,
    view: &ViewBinding,
    parameters: &VbaoParameters,
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
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<VbaoPipelines> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/vbao")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("vbao"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create =
      |fragment: &str, constants: &[(&str, f64)], format: wgpu::TextureFormat| -> XrfResult<wgpu::RenderPipeline> {
        create_checked(device, "vbao", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vbao"),
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
              targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
          })
        })
      };
    let mut searches: Vec<wgpu::RenderPipeline> = Vec::with_capacity(QUALITIES.len());

    for quality in QUALITIES {
      let (slices, steps) = quality.get_vbao_search();

      searches.push(create(
        "fs_search",
        &[("SLICES", slices as f64), ("STEPS", steps as f64)],
        ViewTargets::OCCLUSION,
      )?);
    }

    Ok(VbaoPipelines {
      searches,
      accumulate: create("fs_accumulate", &[], VbaoHistory::FORMAT)?,
      filter: create("fs_filter", &[], ViewTargets::OCCLUSION)?,
    })
  }
}
