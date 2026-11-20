use xrf_error::XrfResult;

use crate::feature::tracing::indirect_trace::indirect_trace_pass::IndirectTracePass;
use crate::feature::tracing::ray_guides::ray_guides_pass::RayGuidesPass;
use crate::feature::tracing::reflections::draw::reflection_pass::ReflectionPass;
use crate::feature::tracing::rtao::rtao_pass::RtaoPass;
use crate::feature::tracing::scene_trace::scene_trace_pass::SceneTracePass;
use crate::feature::tracing::sun_trace::sun_trace_pass::SunTracePass;
use crate::feature::tracing::surroundings::surroundings_pass::SurroundingsPass;
use crate::feature::tracing::view_trace::sway::trace_sway_pass::TraceSwayPass;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
#[cfg(test)]
use crate::shader::generated_modules::GeneratedModules;

/// The passes reflecting and tracing the scene: the screen reflections and the surroundings cube below them, the
/// scene's trace and its sway, the traced sun, indirect light and occlusion, and the guides DLSS reconstructs the rays
/// by.
pub(crate) struct TracingPasses {
  pub reflections: ReflectionPass,
  pub surroundings: SurroundingsPass,
  pub scene_trace: SceneTracePass,
  pub trace_sway: TraceSwayPass,
  pub sun_trace: SunTracePass,
  pub indirect_trace: IndirectTracePass,
  pub rtao: RtaoPass,
  pub ray_guides: RayGuidesPass,
}

impl TracingPasses {
  /// Builds every pass at once, each on a thread of its own.
  ///
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub(crate) fn new(build: PassBuild<'_>) -> XrfResult<Self> {
    let PassBuild {
      source,
      queue,
      view,
      texture,
      sky,
      ..
    } = build;

    std::thread::scope(|threads| {
      let reflections = threads.spawn(move || ReflectionPass::new(queue, source, view, sky));
      let surroundings = threads.spawn(move || SurroundingsPass::new(source, view));
      let scene_trace = threads.spawn(move || SceneTracePass::new(source, view, sky, texture));
      let trace_sway = threads.spawn(move || TraceSwayPass::new(source));
      let sun_trace = threads.spawn(move || SunTracePass::new(source, view, texture));
      let indirect_trace = threads.spawn(move || IndirectTracePass::new(source, view, sky, texture));
      let rtao = threads.spawn(move || RtaoPass::new(source, view, texture));
      let ray_guides = threads.spawn(move || RayGuidesPass::new(source, view));

      Ok(Self {
        reflections: PassBuild::join(reflections)?,
        surroundings: PassBuild::join(surroundings)?,
        scene_trace: PassBuild::join(scene_trace)?,
        trace_sway: PassBuild::join(trace_sway)?,
        sun_trace: PassBuild::join(sun_trace)?,
        indirect_trace: PassBuild::join(indirect_trace)?,
        rtao: PassBuild::join(rtao)?,
        ray_guides: PassBuild::join(ray_guides)?,
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    self.reflections.refresh(source);
    self.surroundings.refresh(source);
    self.scene_trace.refresh(source);
    self.trace_sway.refresh(source);
    self.sun_trace.refresh(source);
    self.indirect_trace.refresh(source);
    self.rtao.refresh(source);
    self.ray_guides.refresh(source);
  }

  /// Declares the generated WGSL its passes draw with: the structs their uniforms and storage share and each module's
  /// bindings, the scene's trace among them, which every traced pass binds.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  #[cfg(test)]
  pub(crate) fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    use crate::feature::tracing::indirect_trace::indirect_trace_parameters::IndirectTraceParameters;
    use crate::feature::tracing::indirect_trace::indirect_trace_uniform::IndirectTraceUniform;
    use crate::feature::tracing::ray_guides::ray_guides_parameters::RayGuidesParameters;
    use crate::feature::tracing::reflections::draw::reflection_parameters::ReflectionParameters;
    use crate::feature::tracing::reflections::reflection_uniform::ReflectionUniform;
    use crate::feature::tracing::reflections::trace::reflection_trace_out_parameters::ReflectionTraceOutParameters;
    use crate::feature::tracing::scene_trace::reflection_scene_parameters::ReflectionSceneParameters;
    use crate::feature::tracing::scene_trace::requests::scene_list_parameters::SceneListParameters;
    use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;
    use crate::feature::tracing::scene_trace::requests::trace_request_parameters::TraceRequestParameters;
    use crate::feature::tracing::scene_trace::requests::water_ray_parameters::WaterRayParameters;
    use crate::feature::tracing::scene_trace::requests::water_rays_uniform::WaterRaysUniform;
    use crate::feature::tracing::sun_trace::sun_trace_parameters::SunTraceParameters;
    use crate::feature::tracing::sun_trace::sun_trace_uniform::SunTraceUniform;
    use crate::feature::tracing::surroundings::surroundings_parameters::SurroundingsParameters;
    use crate::feature::tracing::surroundings::surroundings_uniform::SurroundingsUniform;
    use crate::feature::tracing::view_trace::sway::trace_sway_parameters::TraceSwayParameters;
    use crate::feature::tracing::view_trace::sway::trace_sway_slot::TraceSwaySlot;
    use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;

    modules
      .declare::<TraceSwaySlot>()
      .declare::<TraceRequest>()
      .declare::<WaterRaysUniform>()
      .declare::<SurroundingsUniform>()
      .declare::<SunTraceUniform>()
      .declare::<IndirectTraceUniform>()
      .declare::<ReflectionUniform>()
      .declare_held::<TraceParameters<'_>>();
    modules
      .add::<TraceParameters<'_>>("generated/common/scene_trace")?
      .add::<SunTraceParameters>("generated/frame/sun_trace")?
      .add::<RayGuidesParameters>("generated/frame/ray_guides")?
      .add::<ReflectionParameters<'_>>("generated/frame/reflections")?
      .add::<TraceRequestParameters>("generated/common/trace_requests")?
      .add::<ReflectionSceneParameters<'_>>("generated/frame/reflection_scene")?
      .add::<SceneListParameters>("generated/frame/scene_list")?
      .add::<WaterRayParameters>("generated/frame/water_rays")?
      .add::<ReflectionTraceOutParameters>("generated/frame/reflection_trace_out")?
      .add::<IndirectTraceParameters<'_>>("generated/frame/indirect_trace")?
      .add::<SurroundingsParameters<'_>>("generated/frame/surroundings")?
      .add::<TraceSwayParameters>("generated/static/trace_sway")?;

    Ok(())
  }
}
