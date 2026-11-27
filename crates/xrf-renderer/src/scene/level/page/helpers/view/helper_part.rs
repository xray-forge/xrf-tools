/// Which of the kept objects a build of a view's helpers takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HelperPart {
  /// Every kept object a motion does not carry, none singled out.
  Every,
  /// The kept objects a motion carries, none singled out, built again as they move.
  Moving,
  /// The selected, hovered and sounding objects alone, singled out, drawn over the rest.
  Singled,
}
