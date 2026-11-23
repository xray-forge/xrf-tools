/// What one mention of an info portion does with it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InfoPortionRole {
  /// An `info_*.xml` declares it.
  Declared,
  /// A condlist effect `%+name%`, a dialog's `give_info` or a spawned document gives it to the actor.
  Given,
  /// A dialog file `system.ltx` does not list would give it, but the engine never loads that file.
  GivenUnlisted,
  /// A debug scene of `scripts\scenes.ltx` starts with it, which a developer menu gives to jump into its quest.
  GivenByDebugScene,
  /// A condlist effect `%-name%` or a dialog's `disable_info` takes it away.
  Taken,
  /// A condlist condition `{+name}` or `{-name}`, or a dialog's `has_info` or `dont_has_info`, reads it.
  Read,
  /// A condlist function receives it as a parameter, as `=zat_b29_create_random_infop(name:...)` does, and may give it.
  Passed,
}
