import { IRendererBounds } from "#/contract/scene/renderer-bounds";
import { IRendererProgressive } from "#/contract/scene/renderer-progressive";

/**
 * A range of a geometry's indices drawn with one surface.
 */
export interface IRendererGeometryGroup {
  /** First index of the range. */
  start: number;
  /** Indices in it. */
  count: number;
  /** Which of the object's surfaces draws it, by position in its list. */
  slot: number;
  /** What the range's own vertices span, which it is culled by; measured by the renderer where left out. */
  bounds?: IRendererBounds;
  /** The coarser bands of a progressive mesh, which each place of an instanced draw picks among by its detail. */
  progressive?: IRendererProgressive;
}
