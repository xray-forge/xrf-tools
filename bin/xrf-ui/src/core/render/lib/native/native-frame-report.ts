import { RenderFrameReport, RenderLightsReport, RenderStaticReport } from "@/core/ipc/types/xrf-renderer";

/** Nothing used of a pool that holds nothing. */
const EMPTY_POOL = { capacity: 0, used: 0 } as const;

/** A frame with no level's static draws. */
export const EMPTY_RENDER_STATIC_REPORT: RenderStaticReport = {
  clusters: EMPTY_POOL,
  commands: 0,
  keptClusters: 0,
  keptTriangles: 0,
  lods: EMPTY_POOL,
  occludedClusters: 0,
  occludedTriangles: 0,
  places: EMPTY_POOL,
  rows: EMPTY_POOL,
  slots: EMPTY_POOL,
  surfaceList: EMPTY_POOL,
};

/** A frame with no level's local lights. */
export const EMPTY_RENDER_LIGHTS_REPORT: RenderLightsReport = {
  atlas: EMPTY_POOL,
  dropped: 0,
  excess: 0,
  fullClusters: 0,
  inView: 0,
  shadowed: 0,
};

/** What a viewport reads before its first frame is reported, and after it is let go. */
export const EMPTY_RENDER_FRAME_REPORT: RenderFrameReport = {
  adapter: "",
  backend: "",
  clusters: 0,
  cpuTime: 0,
  frameTime: 0,
  frameTimeMax: 0,
  framesPerSecond: 0,
  height: 0,
  isGpuTimed: false,
  lights: EMPTY_RENDER_LIGHTS_REPORT,
  memory: { scene: 0, textures: 0 },
  passes: [],
  renderHeight: 0,
  renderWidth: 0,
  sectorTime: 0,
  staticDraws: EMPTY_RENDER_STATIC_REPORT,
  triangles: 0,
  width: 0,
};
