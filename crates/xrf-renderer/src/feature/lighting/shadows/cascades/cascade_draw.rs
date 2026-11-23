/// How a cascade due this frame is drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CascadeDraw {
  /// Every caster into its map: while the trees stand still.
  Whole,
  /// What stands still into its static layer, copied into its map, the trees over it.
  Static,
  /// Its static layer copied into its map, the trees over it.
  Swaying,
  /// Its kept layer copied into its map, what moves every frame over it: the trees as last drawn, waiting their turn.
  Kept,
  /// One band of its rows: its static layer's rows restored into its map, the trees over them, so a sway redraw
  /// spreads over `BANDS` frames.
  Banded(u32),
}
