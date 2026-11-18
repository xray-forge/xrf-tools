use crate::compile::level_compile_step_run::LevelCompileStepRun;
use crate::light::bake::light_bake::LightBake;

/// What [`GeometryStep::run`](crate::geometry::level::geometry_step::GeometryStep::run) ran: its own report, and the
/// bake made in the same pass for the light step to report; none for the neutral light.
pub(crate) struct GeometryStepRun {
  pub(crate) run: LevelCompileStepRun,
  pub(crate) bake: Option<LightBake>,
}
