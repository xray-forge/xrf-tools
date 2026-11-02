use crate::feature::tracing::indirect_trace::indirect_traced::IndirectTraced;
use crate::feature::tracing::reflections::reflections_recorded::ReflectionsRecorded;
use crate::feature::weather::wet::wet_targets::WetTargets;

/// What the guides DLSS ray reconstruction reads are worked out from beside the G-buffer: what the wetting wrote, the
/// reflections where they were traced (their trace's hit distances) and the indirect light where it was traced (its
/// rays' hit distances).
#[derive(Clone, Copy)]
pub struct RayGuideInputs {
  pub wet: WetTargets,
  pub reflections: Option<ReflectionsRecorded>,
  pub indirect: Option<IndirectTraced>,
}
