/**
 * How much of something bounded a view sees.
 */
export enum EVisibility {
  /** None of it. */
  OUTSIDE = "outside",
  /** Some of it, or possibly all: the parts inside it have to be asked on their own. */
  INTERSECTS = "intersects",
  /** All of it, and so all of whatever it bounds. */
  INSIDE = "inside",
}
