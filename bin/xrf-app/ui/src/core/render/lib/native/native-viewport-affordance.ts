/**
 * What a drag pressed over a point of a native viewport would do, which decides whether it locks the pointer.
 */
export enum ENativeViewportAffordance {
  /** Moves the view: the camera looks, orbits or pans, with the pointer locked and hidden while it drags. */
  VIEW = "view",
}
