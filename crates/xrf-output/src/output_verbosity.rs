/// Controls which live workflow messages are rendered.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputVerbosity {
  #[default]
  Silent,
  Normal,
  Verbose,
}
