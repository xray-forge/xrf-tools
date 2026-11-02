/**
 * Which of a breakdown's two measurements a view is ordered and drawn by.
 *
 * The two rank differently often enough that neither is the answer on its own: in an Anomaly volume set `ogg` leads by
 * count and comes fifth by bytes, while `geom` is second by bytes with thirty-four files. A proportion bar has to mean
 * one of them, so a view says which rather than leaving a reader to guess.
 */
export enum EStatMeasure {
  BYTES = "bytes",
  COUNT = "count",
}
