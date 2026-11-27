/// What a layer did with a platform event it was offered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GameInputFate {
  /// Not its event: the layers under it are offered it.
  Passed,
  /// Kept from the layers under it.
  Kept,
  /// Kept, and from now the layer has the keyboard and the mouse, as a console or a menu opening does: the layers under
  /// it let go of what they hold, since the releases will never reach them, and the cursor is freed. From a layer that
  /// has them already it is only kept.
  Seized,
  /// Kept, and the layer gives the keyboard and the mouse back, as it closes; from a layer that has them not, only
  /// kept.
  Yielded,
}
