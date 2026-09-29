use serde::Serialize;

/// What kind of problem a finding reports.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EnvironmentRule {
  /// The engine refuses to load it, or loads it as something other than what is written.
  Engine,
  /// A section names another the engine cannot find.
  Reference,
  /// The engine loads it, but against a convention strong enough to be a mistake: the engine's own `! Invalid`
  /// warnings, a value it clamps, a list it pairs item by item written unevenly.
  Convention,
}
