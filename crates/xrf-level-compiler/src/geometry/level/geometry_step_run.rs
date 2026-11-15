use crate::compile::level_compile_step_run::LevelCompileStepRun;
use crate::light::bake::light_bake::LightBake;

/// What the geometry step ran ([`GeometryStep::run`](crate::geometry::level::geometry_step::GeometryStep::run)): its
/// own report, and how the light baked in the same pass went, which the light step reports; none for the neutral light.
pub(crate) struct GeometryStepRun {
  pub(crate) run: LevelCompileStepRun,
  pub(crate) bake: Option<LightBake>,
}
