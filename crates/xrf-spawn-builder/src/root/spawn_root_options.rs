/// How a root spawn is built beyond its levels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpawnRootOptions {
  /// The level the actor starts on, where it is not the actor's own: xrAI's `-start`.
  pub start: Option<String>,
  /// Refuse the build where a restrictor cuts the AI map apart, rather than report it: xrAI's check without
  /// `-no_separator_check`, kept to real cuts.
  pub is_separator_strict: bool,
}
