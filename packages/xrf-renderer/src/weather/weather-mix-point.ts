import { TRendererVector } from "#/contract/renderer-vector";

/**
 * Where a cycle is mixed: at a time of day, seen from a point the level's modifiers reach or not.
 */
export interface IWeatherMixPoint {
  /** Seconds, wrapped into the day. */
  time: number;
  /** In engine space. */
  view: TRendererVector;
}
