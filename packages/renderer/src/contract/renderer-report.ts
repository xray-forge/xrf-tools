import { IRendererCameraPose } from "#/contract/renderer-camera";
import { IRenderFrameCost } from "#/frame/render-frame-cost";

/**
 * What one pass of the frame cost on the GPU.
 */
export interface IRendererPassCost {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds over the report window; zero while no timing has resolved. */
  gpuTime: number;
}

/**
 * How full one pool of the static draw buffers is.
 */
export interface IRendererPoolUse {
  used: number;
  capacity: number;
}

/**
 * What the static draws hold and what occlusion removed from the last frame they were culled for.
 */
export interface IRendererStaticDrawReport {
  /** Slots, a draw each. */
  slots: IRendererPoolUse;
  /** Places instanced draws stand in. */
  places: IRendererPoolUse;
  /** Rows the instance cull tests, a place of one instanced draw each. */
  rows: IRendererPoolUse;
  /** Impostors of clumps of trees, which the LOD cull decides between a clump and its impostor by. */
  lods: IRendererPoolUse;
  /** Times a static draw was refused a slot, place or row by the device's limit and drawn plainly instead. */
  fallbacks: number;
  /** What the frustum kept and the depth pyramid hid in both phases: single draws, instances and triangles. */
  occluded: { draws: number; instances: number; triangles: number };
}

/** Nothing held and nothing culled, which is what a renderer reports before its first cull. */
export const EMPTY_RENDERER_STATIC_DRAW_REPORT: IRendererStaticDrawReport = {
  fallbacks: 0,
  occluded: { draws: 0, instances: 0, triangles: 0 },
  places: { capacity: 0, used: 0 },
  lods: { capacity: 0, used: 0 },
  rows: { capacity: 0, used: 0 },
  slots: { capacity: 0, used: 0 },
};

/**
 * What the renderer says about its frames, a few times a second.
 */
export interface IRendererReport {
  /** Frame pacing and submission cost, measured on the thread that draws. */
  frame: IRenderFrameCost;
  /** GPU cost per pass, in frame order. */
  passes: ReadonlyArray<IRendererPassCost>;
  /** Whether the device grants timestamp queries, without which `passes` stays at zero. */
  isGpuTimed: boolean;
  /** Where the camera was when the report was taken. */
  camera: IRendererCameraPose;
  /** How full the static draws' pools are and what occlusion removed. */
  staticDraws: IRendererStaticDrawReport;
}
