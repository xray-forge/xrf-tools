import {
  RenderFramePhases,
  RenderFrameReport,
  RenderLightsReport,
  RenderParticlesReport,
  RenderStaticReport,
} from "@/core/ipc/types/xrf-renderer";

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

/** A frame with no level's particle systems. */
export const EMPTY_RENDER_PARTICLES_REPORT: RenderParticlesReport = {
  drawn: 0,
  effects: 0,
  particles: 0,
  simulated: 0,
  simulationTime: 0,
};

/** A frame whose render thread spent nothing anywhere. */
export const EMPTY_RENDER_FRAME_PHASES: RenderFramePhases = {
  acquire: 0,
  compose: 0,
  load: 0,
  prepare: 0,
  present: 0,
  record: 0,
  encode: 0,
  submit: 0,
  update: 0,
};

/** What a viewport reads before its first frame is reported, and after it is let go. */
export const EMPTY_RENDER_FRAME_REPORT: RenderFrameReport = {
  adapter: "",
  backend: "",
  cpuTime: 0,
  frameTime: 0,
  frameTimeMax: 0,
  framesPerSecond: 0,
  height: 0,
  isGpuTimed: false,
  lights: EMPTY_RENDER_LIGHTS_REPORT,
  memory: { scene: 0, textures: 0 },
  particles: EMPTY_RENDER_PARTICLES_REPORT,
  passes: [],
  phases: EMPTY_RENDER_FRAME_PHASES,
  renderHeight: 0,
  renderWidth: 0,
  sectorTime: 0,
  staticDraws: EMPTY_RENDER_STATIC_REPORT,
  width: 0,
};
