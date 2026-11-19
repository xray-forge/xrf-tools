/// Which of the wall mark tool's decal chunks the decals are read from, which decides whether each records its size
/// and turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditableWallmarkItems {
  /// `WM_CHUNK_ITEMS`, whose decals keep no size or turn.
  Old,
  /// `WM_CHUNK_ITEMS2`, whose decals record their size and turn.
  Current,
}
