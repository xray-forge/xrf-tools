/// Whether a face a visual takes joins its fast path too, as an opaque surface's do (`fast_path_data`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GeometryFastPath {
  Joined,
  Left,
}
