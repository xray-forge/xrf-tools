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

/**
 * What the local lights came to in the last frame.
 */
export interface IRendererLightsReport {
  /** Lights standing in view, binned and lit: a shadowed one only once its faces are drawn. */
  inView: number;
  /** Of them, the ones drawn with their shadows. */
  shadowed: number;
  /** Lights in view past the most a frame holds, the farthest, left unlit. */
  excessLights: number;
  /** Texels of the shadow atlas the faces hold. */
  atlas: IRendererPoolUse;
  /** What every wanted shadow size is scaled by, below one while the atlas is short of room. */
  shadowScale: number;
  /** Clusters more lights reached than they hold, as the last binning read back found them. */
  fullClusters: number;
  /** Lights left out of a cluster they reached, as the last binning read back found them: one left out of two counts twice. */
  droppedLights: number;
}

/** No lights, which is what a renderer reports before it has any. */
export const EMPTY_RENDERER_LIGHTS_REPORT: IRendererLightsReport = {
  atlas: { capacity: 0, used: 0 },
  droppedLights: 0,
  excessLights: 0,
  fullClusters: 0,
  inView: 0,
  shadowScale: 1,
  shadowed: 0,
};

/**
 * What each pass of the frame cost on the GPU.
 */
export interface IRendererPassTimings {
  /** GPU cost per pass, in frame order. */
  passes: ReadonlyArray<IRendererPassCost>;
  /** Whether passes are being timed: the device grants timestamp queries and the features ask for them. */
  isGpuTimed: boolean;
}

/** No pass timed, for a viewport that has not reported. */
export const EMPTY_RENDERER_PASS_TIMINGS: IRendererPassTimings = { isGpuTimed: false, passes: [] };

/**
 * What the renderer says about its frames, a few times a second.
 */
export interface IRendererReport extends IRendererPassTimings {
  /** Frame pacing and submission cost, measured on the thread that draws. */
  frame: IRenderFrameCost;
  /** Where the camera was when the report was taken. */
  camera: IRendererCameraPose;
  /** How full the static draws' pools are and what occlusion removed. */
  staticDraws: IRendererStaticDrawReport;
  lights: IRendererLightsReport;
}
