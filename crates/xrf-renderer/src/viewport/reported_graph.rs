use xrf_renderer_core::{GraphPassTime, GraphReport, GraphTimer};

/// What a viewport's report reads of the frames' graph: the passes' timer, its own taken from it, what the windows'
/// own passes cost this report round, and what the graph made of the latest described frame.
pub(crate) struct ReportedGraph<'a> {
  pub timer: &'a mut GraphTimer,
  pub composition: &'a [GraphPassTime],
  pub graph: Option<&'a GraphReport>,
}
