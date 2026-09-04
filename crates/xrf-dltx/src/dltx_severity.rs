/// How much a DLTX finding matters.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DltxSeverity {
  /// The engine would load this and say nothing, or nothing unless asked. XRF says it anyway, because a tool that
  /// stays as quiet as the engine is no more use than the engine.
  Warning,
  /// The engine would refuse to start. `Debug.fatal` and the `R_ASSERT` family.
  Error,
}
