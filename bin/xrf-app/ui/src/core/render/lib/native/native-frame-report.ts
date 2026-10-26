import {
  ERenderAntialiasing,
  ERenderBackend,
  ERenderDlssAvailability,
  RenderFramePhases,
  RenderFrameReport,
  RenderGrassReport,
  RenderLightsReport,
  RenderParticlesReport,
  RenderStaticReport,
} from "@xrf/ipc/types/xrf-renderer";

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
const EMPTY_RENDER_LIGHTS_REPORT: RenderLightsReport = {
  atlas: EMPTY_POOL,
  dropped: 0,
  excess: 0,
  fullClusters: 0,
  inView: 0,
  shadowed: 0,
  volumetric: 0,
};

/** A frame with no level's particle systems. */
const EMPTY_RENDER_PARTICLES_REPORT: RenderParticlesReport = {
  drawn: 0,
  effects: 0,
  particles: 0,
  simulated: 0,
  simulationTime: 0,
};

/** A frame with no grass drawn. */
const EMPTY_RENDER_GRASS_REPORT: RenderGrassReport = {
  capacity: 0,
  drawn: 0,
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
  backend: ERenderBackend.D3D12,
  cpuTime: 0,
  cpuTimes: [],
  frameTime: 0,
  frameTimeMax: 0,
  frameTimes: [],
  framesPerSecond: 0,
  gpuTime: null,
  gpuTimes: [],
  graph: null,
  grass: EMPTY_RENDER_GRASS_REPORT,
  groups: [],
  hasRayQueries: false,
  dlss: ERenderDlssAvailability.UNBUILT,
  rayReconstruction: ERenderDlssAvailability.UNBUILT,
  outputWidth: 0,
  outputHeight: 0,
  antialiasing: ERenderAntialiasing.NONE,
  isRayReconstructed: false,
  height: 0,
  isGpuTimed: false,
  lights: EMPTY_RENDER_LIGHTS_REPORT,
  memory: { allocated: null, budget: null, peakAllocated: null, reserved: null, sections: [], total: null, used: null },
  meter: null,
  particles: EMPTY_RENDER_PARTICLES_REPORT,
  passes: [],
  phases: EMPTY_RENDER_FRAME_PHASES,
  renderHeight: 0,
  renderWidth: 0,
  sectorTime: 0,
  staticDraws: EMPTY_RENDER_STATIC_REPORT,
  width: 0,
};
