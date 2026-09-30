import { Nullable } from "@xrf/types";

import { TRendererVector } from "#/contract/renderer-vector";

/**
 * What a pick found under a point of the view: the identity of what is drawn there, never its mesh.
 */
export interface IRendererHit {
  /** The key of the object drawn there. */
  object: string;
  /** The key of the surface drawing it there, one of the object's. */
  surface: string;
  /** Which of the object's places it is, or null for an object its matrix places once. */
  instance: Nullable<number>;
  /** Where the point's ray meets it, in renderer space. */
  point: TRendererVector;
}
