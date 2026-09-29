/**
 * How a weather the consumer hands the renderer takes over from what it showed.
 */
export enum ERendererWeatherTransition {
  /** At once: the first weather of a view, where nothing was shown before it. */
  CUT = "cut",
  /** Faded into over a second and a half: another cycle or source picked by hand. */
  FADE = "fade",
  /** Eased into over a quarter second: one value of a hand-set keyframe changed, so a dragged control keeps up. */
  EASE = "ease",
}
