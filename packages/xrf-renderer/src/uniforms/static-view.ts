/** The camera's views, then each shadow view's: what the lists of kept clusters and the batches' arguments are per. */
export enum EStaticView {
  /** The first phase: what the last frame's depth does not hide. */
  EARLY = 0,
  /** The second: what the first's depth no longer hides of what the last frame's did. */
  LATE = 1,
  /** The first shadow view; shadow view `n` is `SHADOW + n`. */
  SHADOW = 2,
}
