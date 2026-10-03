import { IRendererPoolUse } from "@/core/render/lib/contract/renderer-pool-use";

/**
 * What the static draws hold and what occlusion removed from the last frame they were culled for.
 */
export interface IRendererStaticDrawReport {
  /** Slots, a draw each. */
  slots: IRendererPoolUse;
  /** Places static draws stand in: a single draw's one, an instanced draw's one an instance. */
  places: IRendererPoolUse;
  /** Rows the instance cull tests, a place of one instanced draw each. */
  rows: IRendererPoolUse;
  /** Impostors of clumps of trees, which the LOD cull decides between a clump and its impostor by. */
  lods: IRendererPoolUse;
  /** Clusters static draws are made of, each slot's own run. */
  clusters: IRendererPoolUse;
  /** Entries the batches' regions of each list space hold: the camera's batches', and the shadow views'. */
  lists: { surfaces: IRendererPoolUse; shadows: IRendererPoolUse };
  /** Times a static draw was refused room by the device's limit and drawn plainly instead. */
  fallbacks: number;
  /** Draws the camera's static batches issue a frame: a batch's one a view of its two, whatever it keeps. */
  commands: number;
  /** What the two views kept: clusters and their triangles. */
  kept: { clusters: number; triangles: number };
  /** What the frustum kept and the depth pyramid hid in both phases: clusters and their triangles. */
  occluded: { clusters: number; triangles: number };
}

/** Nothing held and nothing culled, which is what a renderer reports before its first cull. */
export const EMPTY_RENDERER_STATIC_DRAW_REPORT: IRendererStaticDrawReport = {
  clusters: { capacity: 0, used: 0 },
  commands: 0,
  fallbacks: 0,
  kept: { clusters: 0, triangles: 0 },
  lists: { shadows: { capacity: 0, used: 0 }, surfaces: { capacity: 0, used: 0 } },
  lods: { capacity: 0, used: 0 },
  occluded: { clusters: 0, triangles: 0 },
  places: { capacity: 0, used: 0 },
  rows: { capacity: 0, used: 0 },
  slots: { capacity: 0, used: 0 },
};
