use crate::graph::graph_owner::GraphOwner;

/// A timed pass by its owner: one pass's name, or each name of the passes merged into one render pass.
#[derive(Clone, Debug)]
pub(crate) struct TimedPass {
  pub owner: GraphOwner,
  pub passes: Vec<&'static str>,
}

impl TimedPass {
  /// What joins merged passes' names in the name timings report.
  const SEPARATOR: &'static str = " + ";

  /// The name timings and profiles report it by: its passes' names joined.
  pub fn to_name(&self) -> String {
    self.passes.join(Self::SEPARATOR)
  }
}
