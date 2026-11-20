/// A drawable kind beside the static scene, which keys what a consumer learned of a kind while it prepared to the
/// kind's draw it records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawableKind {
  Grass,
}
