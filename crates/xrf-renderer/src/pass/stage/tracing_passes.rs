use xrf_error::XrfResult;

use crate::pass::indirect_trace_pass::IndirectTracePass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::ray_guides_pass::RayGuidesPass;
use crate::pass::reflection_pass::ReflectionPass;
use crate::pass::rtao_pass::RtaoPass;
use crate::pass::scene_trace_pass::SceneTracePass;
use crate::pass::stage::pass_build::PassBuild;
use crate::pass::sun_trace_pass::SunTracePass;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::trace_sway_pass::TraceSwayPass;

/// The passes reflecting and tracing the scene: the screen reflections and the surroundings cube below them, the
/// scene's trace and its sway, the traced sun, indirect light and occlusion, and the guides DLSS reconstructs the rays
/// by.
pub struct TracingPasses {
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
  pub fn new(build: PassBuild<'_>) -> XrfResult<Self> {
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
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.reflections.refresh(source);
    self.surroundings.refresh(source);
    self.scene_trace.refresh(source);
    self.trace_sway.refresh(source);
    self.sun_trace.refresh(source);
    self.indirect_trace.refresh(source);
    self.rtao.refresh(source);
    self.ray_guides.refresh(source);
  }
}
