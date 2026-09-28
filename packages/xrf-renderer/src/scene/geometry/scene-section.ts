import { Sphere } from "three/webgpu";

import { IRendererProgressive } from "#/contract/scene/renderer-progressive";

/**
 * One range of a geometry's indices drawn with one surface, and what it is culled by.
 */
export interface ISceneSection {
  /** First index of the range. */
  start: number;
  /** Indices in it. */
  count: number;
  /** Which of an object's surfaces draws it. */
  slot: number;
  /** What its vertices span, in the geometry's own space. */
  sphere: Sphere;
  /** The coarser bands of a progressive mesh, which an instanced static draw picks among per place. */
  progressive?: IRendererProgressive;
}
