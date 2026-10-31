/// Which of the kept objects a build of a view's helpers takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelperPart {
  /// Every kept object, none singled out.
  Every,
  /// The selected and hovered objects alone, singled out, drawn over every kept object's.
  Singled,
}
