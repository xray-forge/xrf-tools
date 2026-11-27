/// What a hook point is for, as a script reads it beside the point's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HookKind {
  /// Told after the fact, in id order, and changing nothing of it.
  Event,
  /// Asked before an action, and may refuse it.
  Guard,
  /// Handed a value, and may change it.
  Modifier,
  /// A condlist condition (`=name(...)`, `!name(...)`), answering whether it holds.
  Condition,
  /// A condlist effect (`%=name(...)%`), run when its branch is taken.
  Effect,
}
