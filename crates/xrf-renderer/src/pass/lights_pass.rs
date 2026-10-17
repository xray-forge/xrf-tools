use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputeContext, ComputePipelineDescription, PassParameters, RasterContext, RenderPipelineDescription,
};

use crate::context::gpu_requirements::can_trace;
use crate::frame::view_targets::ViewTargets;
use crate::pass::contact_march_permutation::ContactMarchPermutation;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::light_contact_parameters::LightContactParameters;
use crate::pass::light_shadowing_parameters::LightShadowingParameters;
use crate::pass::light_tracing_parameters::LightTracingParameters;
use crate::pass::light_visibility_parameters::LightVisibilityParameters;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;

/// Bins the local lights in view into clusters of the view, marches the contact shadows towards the strongest at half
/// the frame's size, finds how much of each shadowed light every pixel sees, from the shadow atlas or traced, then adds
/// every light reaching each pixel to the light its frame accumulated after the sun.
pub struct LightsPass {
  binning_layout: wgpu::BindGroupLayout,
  layout: wgpu::BindGroupLayout,
  contact_layout: wgpu::BindGroupLayout,
  shadowing_layout: wgpu::BindGroupLayout,
  visibility_layout: wgpu::BindGroupLayout,
  tracing_layout: wgpu::BindGroupLayout,
  /// The scene trace's layout, on a device with ray queries.
  trace_layout: Option<wgpu::BindGroupLayout>,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  pipelines: LightsPipelines,
  /// What the lights pass reads for the visibility while no light in view is shadowed: never read, as no light then
  /// takes a slot.
  unshadowed: wgpu::TextureView,
  /// What the visibility reads for the contact shadows while none are marched: never read, as no light is then marched.
  uncontacted: wgpu::TextureView,
  /// What the visibility and the lights read for the marched lights' weight while none are marched: never read.
  unthresholded: wgpu::TextureView,
  generation: u64,
}

/// The binning, the contact shadows' march, then the visibility and the lights each without contact shadows and with
/// them, which find the slots by them, and the visibility traced, on a device with ray queries.
struct LightsPipelines {
  binning: wgpu::ComputePipeline,
  contact: wgpu::ComputePipeline,
  visibility: [wgpu::ComputePipeline; 2],
  traced: Option<wgpu::ComputePipeline>,
  draws: [wgpu::RenderPipeline; 2],
}

impl LightsPass {
  /// Invocations a binning workgroup runs, as the source declares it.
  pub(crate) const BINNING_WORKGROUP: u32 = 64;
  /// Pixels a side of the square a visibility workgroup covers, as the source declares it.
  pub(crate) const VISIBILITY_WORKGROUP: u32 = 8;

  /// Clusters the view is cut into, sixteen across, nine down and twenty four along it, as `common/light_clusters`
  /// declares them.
  pub const LIGHT_CLUSTERS: u32 = 16 * 9 * 24;

  /// Lights one cluster holds at most; any more reaching it go unlit there.
  pub const LIGHT_CLUSTER_CAPACITY: u32 = 64;

  /// Where the traced visibility binds the texture cache's group, after the scene trace's.
  const TEXTURE_GROUP: u32 = 4;

  /// What the visibility is held in: a share a light, four lights a layer.
  pub const VISIBILITY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  /// Layers of the visibility a pixel's slots fill.
  pub const VISIBILITY_LAYERS: u32 = 2;

  /// What the lights' contact shadows are held in: a light's record and the share of it kept a slot, four a layer.
  pub const CONTACT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

  /// Slots of the lights' contact shadows a layer holds.
  pub const CONTACT_LAYER_SLOTS: u32 = 4;

  /// What the weight a light must reach to be among those marched towards is held in.
  pub const THRESHOLD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let binning_layout: wgpu::BindGroupLayout = LightBinningParameters::create_layout(device);
    let layout: wgpu::BindGroupLayout = LightsParameters::create_layout(device);
    let contact_layout: wgpu::BindGroupLayout = LightContactParameters::create_layout(device);
    let shadowing_layout: wgpu::BindGroupLayout = LightShadowingParameters::create_layout(device);
    let visibility_layout: wgpu::BindGroupLayout = LightVisibilityParameters::create_layout(device);
    let tracing_layout: wgpu::BindGroupLayout = LightTracingParameters::create_layout(device);
    let trace_layout: Option<wgpu::BindGroupLayout> = can_trace(device).then(|| TraceParameters::create_layout(device));
    let pipelines: LightsPipelines = Self::create_pipelines(
      source,
      &binning_layout,
      [view_layout, &layout, texture_layout],
      [&contact_layout, &shadowing_layout, &visibility_layout],
      (&tracing_layout, trace_layout.as_ref()),
    )?;
    let stand_in = |label: &str, format: wgpu::TextureFormat, layers: Option<u32>| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: layers.unwrap_or(1),
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor {
          dimension: Some(if layers.is_some() {
            wgpu::TextureViewDimension::D2Array
          } else {
            wgpu::TextureViewDimension::D2
          }),
          ..Default::default()
        })
    };

    Ok(Self {
      pipelines,
      binning_layout,
      layout,
      contact_layout,
      shadowing_layout,
      visibility_layout,
      tracing_layout,
      trace_layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      unshadowed: stand_in(
        "light visibility none",
        Self::VISIBILITY_FORMAT,
        Some(Self::VISIBILITY_LAYERS),
      ),
      uncontacted: stand_in("light contact none", Self::CONTACT_FORMAT, Some(1)),
      unthresholded: stand_in("light threshold none", Self::THRESHOLD_FORMAT, None),
      generation: source.get_generation(),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(
        source,
        &self.binning_layout,
        [&self.view_layout, &self.layout, &self.texture_layout],
        [&self.contact_layout, &self.shadowing_layout, &self.visibility_layout],
        (&self.tracing_layout, self.trace_layout.as_ref()),
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

  /// What the visibility reads for the contact shadows while none are marched.
  pub fn get_uncontacted(&self) -> &wgpu::TextureView {
    &self.uncontacted
  }

  /// What the visibility and the lights read for the marched lights' weight while none are marched.
  pub fn get_unthresholded(&self) -> &wgpu::TextureView {
    &self.unthresholded
  }

  /// Bins the view's light records into the 16×9×24 clusters the lights pass reads.
  pub fn record_binning(&self, context: &mut ComputeContext<'_>, parameters: &LightBinningParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.binning);
    pass.dispatch_workgroups(Self::LIGHT_CLUSTERS.div_ceil(Self::BINNING_WORKGROUP), 1, 1);
  }

  /// Marches the contact shadows towards each texel's strongest lights into a target `size` texels across, half the
  /// frame's.
  pub fn record_contact(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    (parameters, contact): (&LightsParameters<'_>, &LightContactParameters),
    size: (u32, u32),
  ) {
    context.bind(parameters);
    context.bind(contact);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines.contact);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(
      size.0.div_ceil(Self::VISIBILITY_WORKGROUP),
      size.1.div_ceil(Self::VISIBILITY_WORKGROUP),
      1,
    );
  }

  /// Finds how much of each shadowed light every pixel of a `size` view sees, reading the contact shadows marched where
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
      size.0.div_ceil(Self::VISIBILITY_WORKGROUP),
      size.1.div_ceil(Self::VISIBILITY_WORKGROUP),
      1,
    );
  }

  /// Traces how much of each shadowed light every pixel of a `size` view sees, the contact shadows not marched; nothing
  /// on a device without ray queries.
  pub fn record_tracing(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    (parameters, tracing): (&LightsParameters<'_>, &LightTracingParameters),
    (trace, textures): (&TraceParameters<'_>, &wgpu::BindGroup),
    size: (u32, u32),
  ) {
    let Some(pipeline) = &self.pipelines.traced else {
      return;
    };

    context.bind(parameters);
    context.bind(tracing);
    context.bind(trace);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(Self::TEXTURE_GROUP, textures, &[]);
    pass.dispatch_workgroups(
      size.0.div_ceil(Self::VISIBILITY_WORKGROUP),
      size.1.div_ceil(Self::VISIBILITY_WORKGROUP),
      1,
    );
  }

  /// Adds every light to the light target, with the contact shadows' marched shares where they were marched
  /// (`is_contact`).
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
    source: PipelineSource<'_>,
    binning_layout: &wgpu::BindGroupLayout,
    [view_layout, layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
    [contact_layout, shadowing_layout, visibility_layout]: [&wgpu::BindGroupLayout; 3],
    (tracing_layout, trace_layout): (&wgpu::BindGroupLayout, Option<&wgpu::BindGroupLayout>),
  ) -> XrfResult<LightsPipelines> {
    let binning: wgpu::ComputePipeline = source.get_compute_pipeline(
      &ComputePipelineDescription::new("light binning", "frame/light_binning", "cs_bin")
        .with_bind_group_layouts(&[Some(binning_layout)]),
    )?;
    let contact: wgpu::ComputePipeline = source.get_compute_pipeline(
      &ComputePipelineDescription::new("light contact", "frame/light_contact", "cs_light_contact")
        .with_bind_group_layouts(&[Some(view_layout), Some(layout), None, Some(contact_layout)]),
    )?;
    let permutation = |is_contact: bool| ContactMarchPermutation {
      contact_marched: is_contact,
    };
    let create_visibility = |is_contact: bool| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(
          "light visibility",
          ContactMarchPermutation::VISIBILITY_MODULE,
          "cs_visibility",
        )
        .with_permutation(&permutation(is_contact))
        .with_bind_group_layouts(&[Some(view_layout), Some(layout), None, Some(shadowing_layout)]),
      )
    };
    let traced: Option<wgpu::ComputePipeline> = trace_layout
      .map(|trace_layout| {
        source.get_compute_pipeline(
          &ComputePipelineDescription::new("light tracing", "frame/light_tracing", "cs_visibility_traced")
            .with_bind_group_layouts(&[
              Some(view_layout),
              Some(layout),
              Some(tracing_layout),
              Some(trace_layout),
              Some(texture_layout),
            ]),
        )
      })
      .transpose()?;
    let additive: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };
    let create_draw = |is_contact: bool| -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new("lights", ContactMarchPermutation::LIGHTS_MODULE, "vs_fullscreen")
          .with_fragment("fs_lights")
          .with_permutation(&permutation(is_contact))
          .with_bind_group_layouts(&[
            Some(view_layout),
            Some(layout),
            Some(texture_layout),
            Some(visibility_layout),
          ])
          .with_target(wgpu::ColorTargetState {
            format: ViewTargets::LIGHT,
            blend: Some(wgpu::BlendState {
              color: additive,
              alpha: additive,
            }),
            write_mask: wgpu::ColorWrites::ALL,
          }),
      )
    };

    Ok(LightsPipelines {
      binning,
      contact,
      visibility: [create_visibility(false)?, create_visibility(true)?],
      traced,
      draws: [create_draw(false)?, create_draw(true)?],
    })
  }
}
