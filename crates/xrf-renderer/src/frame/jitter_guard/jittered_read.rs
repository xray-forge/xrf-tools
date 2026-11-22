/// A pass after a view's temporal resolve that reads a texture still carrying the view's jitter, by their labels.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct JitteredRead {
  pub pass: String,
  pub texture: String,
}
