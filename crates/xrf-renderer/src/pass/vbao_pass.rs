use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::frame::indirect_light_history::IndirectLightHistory;
use crate::frame::vbao_history::VbaoHistory;
use crate::frame::view_targets::ViewTargets;
use crate::pass::bitmask_search::BitmaskSearch;
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

/// The visibility-bitmask search at half the frame's size, for VBAO's occlusion, the indirect light or both:
/// searched, accumulated over frames into the view's histories, then filtered into what combine reads.
pub struct VbaoPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipelines: VbaoPipelines,
  /// What a stage reads in place of a history where there is none: one texel, all visible and nothing drawn.
  empty: wgpu::TextureView,
  /// What reads in place of light where none is gathered: one texel of none.
  dark: wgpu::TextureView,
  generation: u64,
}

/// The search a quality, unlit and gathering light; the accumulations; the filters by what they yield; and the light
/// copied for the search.
struct VbaoPipelines {
  searches: Vec<wgpu::RenderPipeline>,
  lit_searches: Vec<wgpu::RenderPipeline>,
  accumulate: wgpu::RenderPipeline,
  lit_accumulate: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
  lit_filter: wgpu::RenderPipeline,
  light_filter: wgpu::RenderPipeline,
  light_source: wgpu::RenderPipeline,
}

impl VbaoPass {
  /// The light copied for the search, gathered and filtered: colour, then distance along the view where it says.
  pub const LIGHT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

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
    let texel = |label: &str, value: [u8; 4]| -> wgpu::TextureView {
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          },
          Default::default(),
          &value,
        )
        .create_view(&Default::default())
    };

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      empty: texel("vbao history empty", [u8::MAX, 0, 0, 0]),
      dark: texel("indirect light none", [0; 4]),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("VBAO rejected, searching with the last one: {error}"),
      }
    }
  }

  /// The texel a stage reads where no history is bound.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// The texel read in place of light where none is gathered.
  pub fn get_dark(&self) -> &wgpu::TextureView {
    &self.dark
  }

  /// Copies the light the frame's surfaces leave, at the search's size.
  pub fn record_light_source(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &VbaoParameters) {
    Self::draw(context, &self.pipelines.light_source, view, parameters);
  }

  /// Searches the frame's pixels at `quality` into the targets the pass draws into, gathering the light while lit.
  pub fn record_search(
    &self,
    context: &mut RasterContext<'_>,
    (quality, is_lit): (RenderAmbientOcclusionQuality, bool),
    view: &ViewBinding,
    parameters: &VbaoParameters,
  ) {
    let search: usize = QUALITIES.iter().position(|it| *it == quality).unwrap_or(2);
    let searches: &[wgpu::RenderPipeline] = if is_lit {
      &self.pipelines.lit_searches
    } else {
      &self.pipelines.searches
    };

    Self::draw(context, &searches[search], view, parameters);
  }

  /// Blends the search with the last frame's accumulation into this frame's, the light's too while lit.
  pub fn record_accumulate(
    &self,
    context: &mut RasterContext<'_>,
    is_lit: bool,
    view: &ViewBinding,
    parameters: &VbaoParameters,
  ) {
    let pipeline: &wgpu::RenderPipeline = if is_lit {
      &self.pipelines.lit_accumulate
    } else {
      &self.pipelines.accumulate
    };

    Self::draw(context, pipeline, view, parameters);
  }

  /// Filters the accumulation, or the search alone, into what combine reads: the occlusion, the light, or both.
  pub fn record_filter(
    &self,
    context: &mut RasterContext<'_>,
    search: BitmaskSearch,
    view: &ViewBinding,
    parameters: &VbaoParameters,
  ) {
    let pipeline: &wgpu::RenderPipeline = match (search.is_occluding, search.is_lit) {
      (true, true) => &self.pipelines.lit_filter,
      (false, true) => &self.pipelines.light_filter,
      _ => &self.pipelines.filter,
    };

    Self::draw(context, pipeline, view, parameters);
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
      |fragment: &str, constants: &[(&str, f64)], formats: &[wgpu::TextureFormat]| -> XrfResult<wgpu::RenderPipeline> {
        let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|it| Some((*it).into())).collect();

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
    let occluded: &[wgpu::TextureFormat] = &[ViewTargets::OCCLUSION];
    let lit: &[wgpu::TextureFormat] = &[ViewTargets::OCCLUSION, Self::LIGHT];
    let mut searches: Vec<wgpu::RenderPipeline> = Vec::with_capacity(QUALITIES.len());
    let mut lit_searches: Vec<wgpu::RenderPipeline> = Vec::with_capacity(QUALITIES.len());

    for quality in QUALITIES {
      let (slices, steps) = quality.get_vbao_search();
      let constants: &[(&str, f64)] = &[("SLICES", slices as f64), ("STEPS", steps as f64)];
      let lit_constants: &[(&str, f64)] = &[
        ("SLICES", slices as f64),
        ("STEPS", steps as f64),
        ("FAR_STEPS", quality.get_indirect_steps() as f64),
      ];

      searches.push(create("fs_search", constants, occluded)?);
      lit_searches.push(create("fs_search_lit", lit_constants, lit)?);
    }

    Ok(VbaoPipelines {
      searches,
      lit_searches,
      accumulate: create("fs_accumulate", &[], &[VbaoHistory::FORMAT])?,
      lit_accumulate: create(
        "fs_accumulate_lit",
        &[],
        &[VbaoHistory::FORMAT, IndirectLightHistory::FORMAT],
      )?,
      filter: create("fs_filter", &[], occluded)?,
      lit_filter: create("fs_filter_lit", &[], lit)?,
      light_filter: create("fs_filter_light", &[], &[Self::LIGHT])?,
      light_source: create("fs_light_source", &[], &[Self::LIGHT])?,
    })
  }
}
