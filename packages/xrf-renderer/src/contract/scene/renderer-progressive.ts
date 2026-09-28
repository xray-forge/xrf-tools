/**
 * The bands a progressive mesh's group is drawn in, one of which each place of an instanced draw picks by its detail:
 * a few of the engine's slide windows, the group's own range the whole detail (`FTreeVisual_PM::Render`).
 */
export interface IRendererProgressive {
  /** Windows the engine's table has, which a place's detail picks one of before its band is taken. */
  windows: number;
  /**
   * A range of the geometry's indices per band, the whole detail first. Band `b` is window
   * `floor(b * windows / bands.length)`, and a place draws band `floor(window * bands.length / windows)`: never
   * coarser than the window the engine would draw it with.
   */
  bands: ReadonlyArray<{ start: number; count: number }>;
}
