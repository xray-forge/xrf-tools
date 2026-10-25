/// Controls which live workflow messages are rendered.
///
/// Verbosity gates chatter only: `Silent` mutes headings, successes, warnings, info, and verbose
/// detail, while `error` and `failure` always reach the output so a failing run is never silent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputVerbosity {
  #[default]
  Silent,
  Normal,
  Verbose,
}
