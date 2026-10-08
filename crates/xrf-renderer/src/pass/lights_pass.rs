use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::light_shadowing_parameters::LightShadowingParameters;
use crate::pass::light_visibility_parameters::LightVisibilityParameters;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::view_binding::ViewBinding;

/// Clusters the view is cut into, and invocations a binning workgroup runs, as the source declare them.
const LIGHT_CLUSTERS: u32 = 16 * 9 * 24;
const BINNING_WORKGROUP: u32 = 64;

/// Pixels a side of the square a visibility workgroup covers, as the source declares it.
const VISIBILITY_WORKGROUP: u32 = 8;

/// Bins the local lights in view into clusters of the view, finds how much of each shadowed light every pixel sees,
/// then adds every light reaching each pixel to the light its frame accumulated after the sun.
pub struct LightsPass {
  binning_layout: wgpu::BindGroupLayout,
  layout: wgpu::BindGroupLayout,
  shadowing_layout: wgpu::BindGroupLayout,
  visibility_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  pipelines: LightsPipelines,
  /// What the lights pass reads for the visibility while no light in view is shadowed: never read, as no light then
  /// takes a slot.
  unshadowed: wgpu::TextureView,
  generation: u64,
}

/// The binning, then the visibility and the lights each without contact shadows and with them, which find the slots
/// by them.
struct LightsPipelines {
  binning: wgpu::ComputePipeline,
  visibility: [wgpu::ComputePipeline; 2],
  draws: [wgpu::RenderPipeline; 2],
}

impl LightsPass {
  /// What the visibility is held in: a share a light, four lights a layer.
  pub const VISIBILITY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  /// Layers of the visibility a pixel's slots fill.
  pub const VISIBILITY_LAYERS: u32 = 2;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let binning_layout: wgpu::BindGroupLayout = LightBinningParameters::create_layout(device);
    let layout: wgpu::BindGroupLayout = LightsParameters::create_layout(device);
    let shadowing_layout: wgpu::BindGroupLayout = LightShadowingParameters::create_layout(device);
    let visibility_layout: wgpu::BindGroupLayout = LightVisibilityParameters::create_layout(device);
    let pipelines: LightsPipelines = Self::create_pipelines(
      device,
      source,
      &binning_layout,
      [view_layout, &layout, texture_layout],
      [&shadowing_layout, &visibility_layout],
    )?;

    Ok(Self {
      pipelines,
      binning_layout,
      layout,
      shadowing_layout,
      visibility_layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      unshadowed: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("light visibility none"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: Self::VISIBILITY_LAYERS,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::VISIBILITY_FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor {
          dimension: Some(wgpu::TextureViewDimension::D2Array),
          ..Default::default()
        }),
      generation: source.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(
        device,
        source,
        &self.binning_layout,
        [&self.view_layout, &self.layout, &self.texture_layout],
        [&self.shadowing_layout, &self.visibility_layout],
      ) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Lights rejected, lighting with the last one: {error}"),
      }
    }
  }

  /// What the lights pass reads for the visibility while no light in view is shadowed.
  pub fn get_unshadowed(&self) -> &wgpu::TextureView {
    &self.unshadowed
  }

  pub fn record_binning(&self, context: &mut ComputeContext<'_>, parameters: &LightBinningParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.binning);
    pass.dispatch_workgroups(LIGHT_CLUSTERS.div_ceil(BINNING_WORKGROUP), 1, 1);
  }

  /// Finds how much of each shadowed light every pixel of a `size` view sees, marching contact shadows where
  /// `is_contact`.
  pub fn record_visibility(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    (parameters, shadowing): (&LightsParameters<'_>, &LightShadowingParameters<'_>),
    (size, is_contact): ((u32, u32), bool),
  ) {
    context.bind(parameters);
    context.bind(shadowing);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.visibility[usize::from(is_contact)]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(
      size.0.div_ceil(VISIBILITY_WORKGROUP),
      size.1.div_ceil(VISIBILITY_WORKGROUP),
      1,
    );
  }

  pub fn record_draw(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, visibility): (&LightsParameters<'_>, &LightVisibilityParameters),
    (textures, is_contact): (&wgpu::BindGroup, bool),
  ) {
    context.bind(parameters);
    context.bind(visibility);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.draws[usize::from(is_contact)]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    binning_layout: &wgpu::BindGroupLayout,
    [view_layout, layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
    [shadowing_layout, visibility_layout]: [&wgpu::BindGroupLayout; 2],
  ) -> XrfResult<LightsPipelines> {
    let binning_module: wgpu::ShaderModule = source.get_module("frame/light_binning")?;
    let visibility_module: wgpu::ShaderModule = source.get_module("frame/light_visibility")?;
    let module: wgpu::ShaderModule = source.get_module("frame/lights")?;
    let binning_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("light binning"),
        bind_group_layouts: &[Some(binning_layout)],
        ..Default::default()
      });
    let visibility_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("light visibility"),
        bind_group_layouts: &[Some(view_layout), Some(layout), None, Some(shadowing_layout)],
        ..Default::default()
      });
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("lights"),
      bind_group_layouts: &[
        Some(view_layout),
        Some(layout),
        Some(texture_layout),
        Some(visibility_layout),
      ],
      ..Default::default()
    });
    let binning: wgpu::ComputePipeline = create_checked(device, "light binning", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("light binning"),
        layout: Some(&binning_pipeline_layout),
        module: &binning_module,
        entry_point: Some("bin"),
        compilation_options: Default::default(),
        cache: source.get_driver_cache(),
      })
    })?;
    let constants = |is_contact: bool| [("CONTACT_MARCHED", f64::from(u8::from(is_contact)))];
    let create_visibility = |is_contact: bool| -> XrfResult<wgpu::ComputePipeline> {
      create_checked(device, "light visibility", || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some("light visibility"),
          layout: Some(&visibility_pipeline_layout),
          module: &visibility_module,
          entry_point: Some("cs_visibility"),
          compilation_options: wgpu::PipelineCompilationOptions {
            constants: &constants(is_contact),
            ..Default::default()
          },
          cache: source.get_driver_cache(),
        })
      })
    };
    let additive: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };
    let create_draw = |is_contact: bool| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "lights", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("lights"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_lights"),
            compilation_options: wgpu::PipelineCompilationOptions {
              constants: &constants(is_contact),
              ..Default::default()
            },
            targets: &[Some(wgpu::ColorTargetState {
              format: ViewTargets::LIGHT,
              blend: Some(wgpu::BlendState {
                color: additive,
                alpha: additive,
              }),
              write_mask: wgpu::ColorWrites::ALL,
            })],
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: source.get_driver_cache(),
        })
      })
    };

    Ok(LightsPipelines {
      binning,
      visibility: [create_visibility(false)?, create_visibility(true)?],
      draws: [create_draw(false)?, create_draw(true)?],
    })
  }
}
