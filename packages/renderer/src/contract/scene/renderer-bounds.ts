import { TRendererVector } from "#/contract/renderer-vector";

/**
 * A sphere enclosing a geometry's positions, as whoever made them measured it.
 */
export interface IRendererBounds {
  center: TRendererVector;
  radius: number;
}
