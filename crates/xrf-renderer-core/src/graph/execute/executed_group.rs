use std::time::Duration;

/// What recording one encode group cost the CPU: its passes' recording, and wgpu's encoding of it in `finish`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutedGroup {
  pub name: &'static str,
  pub record: Duration,
  pub finish: Duration,
}
