// Auto-generated rust bindings. Do not edit it manually.

import { WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { SectorSkip } from "@/core/ipc/types/xrf-visual";

/** How hard the ambient occlusion searches: XeGTAO's presets. */
export enum ERenderAmbientOcclusionQuality {
  /** One direction, two steps each way. */
  LOW = "low",
  /** Two directions, two steps. */
  MEDIUM = "medium",
  /** Three directions, three steps, `Base`'s choice. */
  HIGH = "high",
  /** Six directions, three steps. */
  ULTRA = "ultra",
}

/** Every `ERenderAmbientOcclusionQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAmbientOcclusionQuality = `${ERenderAmbientOcclusionQuality}`;

/**
 * Ambient occlusion from the depth of the frame, GTAO as XeGTAO computes it at half resolution: it darkens the
 * hemisphere and ambient light over the baked hemisphere occlusion, as the engine's SSAO does.
 */
export type RenderAmbientOcclusionSettings = {
  isEnabled: boolean;
  /** Metres around a point that what stands there occludes it from. */
  radius: number | null;
  /** How dark the occlusion goes: one XeGTAO's own curve, zero none, two its square. */
  strength: number | null;
  quality: RenderAmbientOcclusionQuality;
};

/** How a viewport's finished frame has its edges smoothed. */
export enum ERenderAntialiasing {
  /** Every edge as drawn. */
  NONE = "none",
  /** One pass over the drawn frame's edges, softest and cheapest. */
  FXAA = "fxaa",
  /** Three passes over the drawn frame's edges, crisp and stable. */
  SMAA = "smaa",
  /** Temporal: every frame's samples jittered within the pixel and resolved with the frames before. */
  TAA = "taa",
  /** FSR 2: temporal as TAA, with each surface's motion, disocclusion, reactivity and thin-feature locks. */
  FSR2 = "fsr2",
}

/** Every `ERenderAntialiasing` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAntialiasing = `${ERenderAntialiasing}`;

/** What the weather lights a scene with now, its keyframes blended, as the passes bind it. */
export type RenderAppliedEnvironment = {
  /** The direction sunlight travels, in renderer space. */
  sunDirection: [number | null, number | null, number | null];
  /** The sun's colour, times its light scale. */
  sunColor: [number | null, number | null, number | null];
  /** The ambient as combine binds it: doubled, floored, times its light scale. */
  ambient: [number | null, number | null, number | null];
  /** The hemisphere as combine binds it. */
  hemisphere: [number | null, number | null, number | null];
  /** Distance fog, or none. */
  fog: RenderAppliedFog | null;
  /** `rain_density`, zero for a dry sky. */
  rainDensity: number | null;
  /** `trees_amplitude`, zero for trees standing still. */
  treeSway: number | null;
  /** `water_intensity`. */
  waterIntensity: number | null;
};

/** Distance fog as drawn. */
export type RenderAppliedFog = {
  color: [number | null, number | null, number | null];
  /** Metres to where it is total, or the far plane where that is nearer. */
  distance: number | null;
  /** How near the camera it starts, a share of its distance. */
  density: number | null;
};

/** The grass as planted. */
export type RenderAppliedGrass = {
  /** Metres around the camera it is planted to, in whole slots. */
  radius: number | null;
  /** How far apart a slot's candidates stand, held to the engine's bounds. */
  density: number | null;
  height: number | null;
  /** Tufts the lists hold at most. */
  tufts: number;
  /** Tufts the radius and density would plant: more than `tufts` where a GPU buffer cannot hold them all. */
  wanted: number;
};

/** What a viewport's frames are drawn with, as the renderer resolved what it was asked: sent as it changes. */
export type RenderAppliedReport = {
  /** What smooths the finished frame: none where a target other than the frame is shown. */
  antialiasing: RenderAntialiasing;
  /** How much smaller the scene is drawn than the viewport, native for anything but a level. */
  renderScale: RenderScale;
  /** The sun's shadow, or none where no cascade is drawn. */
  shadows: RenderAppliedShadows | null;
  /** The screen's ambient occlusion, or none where it is off or the scene is unlit. */
  ambientOcclusion: RenderAmbientOcclusionQuality | null;
  /** The local lights, or none where they are off. */
  lights: RenderLightsSettings | null;
  /** The grass planted, or none where none is. */
  grass: RenderAppliedGrass | null;
  /** Whether the level's water is drawn. */
  isWater: boolean;
  /** What the weather lights the scene with now, or none for an asset viewer's rig. */
  environment: RenderAppliedEnvironment | null;
  /**
   * The lens flare whose sprite the sky draws, its `suns.ltx` section: the sun or the moon; none where the sky shows
   * neither.
   */
  sun: string | null;
};

/** The sun's shadow as drawn. */
export type RenderAppliedShadows = {
  /** Each cascade's width in metres, nearest first, as many as are drawn. */
  cascades: Array<number | null>;
  /** Texels each cascade's map is across, held to what the device allows. */
  resolution: number;
  /** Texels the filter reaches each way. */
  filter: number;
};

/**
 * How an asset viewer lights what it shows, in place of a level's weather: one light from a direction and a uniform
 * ambient, each a colour scaled by an intensity.
 */
export type RenderAssetLighting = {
  /** Degrees above the horizon the light sits at, ninety directly overhead. */
  sunElevation: number | null;
  /** Degrees around the vertical, zero looking along renderer `+z`. */
  sunAzimuth: number | null;
  sunIntensity: number | null;
  /** The light's colour, each channel zero to one. */
  sunColor: [number | null, number | null, number | null];
  ambientIntensity: number | null;
  ambientColor: [number | null, number | null, number | null];
};

/**
 * A checkerboard where nothing was drawn, as the one behind a picture with alpha: the backdrop and a second colour in
 * squares.
 */
export type RenderBackdropSquares = {
  /** The second colour, each channel zero to one. */
  color: [number | null, number | null, number | null];
  /** Side of one square, in device pixels of the viewport. */
  size: number | null;
};

/** Every `kind` the `RenderCamera` union is told apart by, so a switch or a comparison names one. */
export enum ERenderCamera {
  /** Flies free, turned by a drag and moved by the keys, as a level is walked. */
  FLY = "fly",
  /** Orbits a target, as a model or texture preview does. */
  ORBIT = "orbit",
}

/**
 * A camera a viewport is driven by, as its consumer describes it.
 *
 * Described again from the same start, a camera keeps where it has been moved and takes only the rest.
 */
export type RenderCamera =
  /** Flies free, turned by a drag and moved by the keys, as a level is walked. */
  | {
      kind: "fly";
      /** Where the camera starts, and returns to on reset. */
      position: [number | null, number | null, number | null];
      /** What it looks at from there. */
      target: [number | null, number | null, number | null];
      /** Vertical field of view, in degrees. */
      fieldOfView: number | null;
      near: number | null;
      far: number | null;
      /** Metres a second at a walk. */
      speed: number | null;
      /** Times the speed while the boost key is held. */
      boost: number | null;
      /** Radians of turn per CSS pixel dragged. */
      sensitivity: number | null;
    }
  /** Orbits a target, as a model or texture preview does. */
  | {
      kind: "orbit";
      /** Where the camera starts, and returns to on reset. */
      position: [number | null, number | null, number | null];
      /** What the camera looks at and turns around. */
      target: [number | null, number | null, number | null];
      /** Vertical field of view, in degrees. */
      fieldOfView: number | null;
      near: number | null;
      far: number | null;
    };

/** Every `kind` the `RenderCameraCommand` union is told apart by, so a switch or a comparison names one. */
export enum ERenderCameraCommand {
  /** Back to where the camera started. */
  RESET = "reset",
  /** Towards the target or away from it, by a multiplier on the distance: above one moves away. */
  DOLLY = "dolly",
}

/** What a consumer can ask of the camera it described. */
export type RenderCameraCommand =
  /** Back to where the camera started. */
  | { kind: "reset" }
  /** Towards the target or away from it, by a multiplier on the distance: above one moves away. */
  | { kind: "dolly"; step: number | null };

/** Where a camera is and the point it looks at. */
export type RenderCameraPose = {
  position: [number | null, number | null, number | null];
  target: [number | null, number | null, number | null];
};

/** An opaque colour as CSS states it, eight bits a channel in sRGB. */
export type RenderColor = {
  r: number;
  g: number;
  b: number;
};

/** Which picture a viewport shows: its finished frame, or one of the targets the frame was built from. */
export enum ERenderDebugView {
  /** The finished frame. */
  FINAL = "final",
  /** The G-buffer's albedo, raw. */
  ALBEDO = "albedo",
  /** The gloss the albedo target carries in its alpha. */
  GLOSS = "gloss",
  /** The view space normal, remapped to colour. */
  NORMAL = "normal",
  /** The baked hemisphere occlusion. */
  HEMI = "hemi",
  /** The baked sun occlusion. */
  SUN = "sun",
  /** The lighting model slice, `(class + 0.5) / 4`. */
  MATERIAL = "material",
  /** View distance, logarithmic: near dark, far light. */
  DEPTH = "depth",
  /** What the sun and the lights accumulated. */
  LIGHT = "light",
  /** The screen's occlusion, white where it is off. */
  AMBIENT_OCCLUSION = "ambientOcclusion",
  /** How far each surface moved on the screen since the last frame: red across, green down, grey still. */
  MOTION = "motion",
}

/** Every `ERenderDebugView` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDebugView = `${ERenderDebugView}`;

/**
 * The engine's exposure (`r2_tonemap`): the frame's average luminance measured every frame, and the scale the tonemap
 * multiplies by moved towards `middle_gray / luminance` at the adaptation's rate.
 */
export type RenderExposureSettings = {
  /** Off, the tonemap multiplies by one, the engine's answer at noon. */
  isEnabled: boolean;
  /** `r2_tonemap_amount`: how far from no adaptation towards the whole of it. */
  amount: number | null;
  /** `r2_tonemap_middlegray`: the luminance the frame is brought towards. */
  middleGray: number | null;
  /** `r2_tonemap_lowlum`: what the luminance is floored at, so a black frame is not brightened without end. */
  lowLuminance: number | null;
  /** `r2_tonemap_adaptation`: how fast the scale follows the frame. */
  adaptation: number | null;
};

/** How often a viewport's frames are drawn and how they are presented. */
export type RenderFrameRate = {
  /**
   * Frames a second drawn at most, or none for no cap: presented at the display's refresh while `is_vsync`, and as
   * fast as a frame is drawn otherwise.
   */
  limit: number | null;
  /** Whether a frame waits for a refresh of the display to be presented, which a faster limit stops at. */
  isVsync: boolean;
};

/** What a viewport's recent frames cost, reported a few times a second while it draws. */
export type RenderFrameReport = {
  /** Frames presented a second over the reported span. */
  framesPerSecond: number | null;
  /** Mean milliseconds between presented frames. */
  frameTime: number | null;
  /** Longest milliseconds between two presented frames. */
  frameTimeMax: number | null;
  /** Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting. */
  cpuTime: number | null;
  /** Drawn width, in device pixels. */
  width: number;
  /** Drawn height, in device pixels. */
  height: number;
  /** The scene's width as rendered, smaller than the drawn one where it is upscaled. */
  renderWidth: number;
  /** And its height. */
  renderHeight: number;
  /** The graphics API drawn with. */
  backend: string;
  /** The GPU drawn on. */
  adapter: string;
  /** Whether its passes were timed on the GPU over the span. */
  isGpuTimed: boolean;
  /** What each pass cost on the GPU, in frame order; none while untimed. */
  passes: Array<RenderPassCost>;
  /** The level's static draws' pools and cull, empty without a level. */
  staticDraws: RenderStaticReport;
  /** The level's local lights, empty without a level. */
  lights: RenderLightsReport;
  /** The level's particle systems, empty without a level. */
  particles: RenderParticlesReport;
  /** Milliseconds the last sector taken in took to put into the scene, on the render thread. */
  sectorTime: number | null;
  /** What the renderer holds on the GPU. */
  memory: RenderMemoryReport;
};

/**
 * The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
 * G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
 */
export type RenderGrassSettings = {
  isEnabled: boolean;
  /** How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`. */
  density: number | null;
  /** Whole metres around the camera grass is planted to: `r__detail_radius`. */
  radius: number | null;
  /** What every planted tuft is scaled by: `r__detail_height`. */
  height: number | null;
};

/**
 * Anomaly's `img_corrections`, which `combine_2` applies to the finished frame: `r__exposure`, `r__gamma`,
 * `r__saturation` and `r__color_grading`.
 */
export type RenderImageCorrections = {
  /** What the frame is multiplied by. */
  exposure: number | null;
  /** The power the frame is raised to, inverted. */
  gamma: number | null;
  /** How far from grey towards the frame's own colour: one leaves it. */
  saturation: number | null;
  /** The colour the mid tones are graded towards; black grades nothing. */
  grading: [number | null, number | null, number | null];
};

/** One gesture over a viewport, as much of the browser's event as crosses. */
export type RenderInputEvent = {
  kind: RenderInputKind;
  pointerId: number;
  isPrimary: boolean;
  button: number;
  buttons: number;
  /** CSS pixels from the viewport element's left edge. */
  x: number | null;
  /** CSS pixels from the viewport element's top edge. */
  y: number | null;
  deltaX: number | null;
  deltaY: number | null;
  deltaMode: number;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  /** `KeyboardEvent.code` for a key, empty for anything else: the key's place, so `W` is `W` on azerty too. */
  code: string;
};

/** The gestures a viewport is told about, named as the browser names them. */
export enum ERenderInputKind {
  CONTEXT_MENU = "contextmenu",
  POINTER_CANCEL = "pointercancel",
  POINTER_DOWN = "pointerdown",
  POINTER_MOVE = "pointermove",
  POINTER_UP = "pointerup",
  WHEEL = "wheel",
  KEY_DOWN = "keydown",
  KEY_UP = "keyup",
  /** The viewport lost focus, so no key it heard go down is still held. */
  BLUR = "blur",
}

/** Every `ERenderInputKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderInputKind = `${ERenderInputKind}`;

/** Every `kind` the `RenderLevelHit` union is told apart by, so a switch or a comparison names one. */
export enum ERenderLevelHit {
  /** A surface the level compiled. */
  SURFACE = "surface",
  /** An object the level's spawn places. */
  SPAWN = "spawn",
}

/** What of a level is drawn under a point of a viewport, and where the ray from the eye met it, in renderer space. */
export type RenderLevelHit =
  /** A surface the level compiled. */
  | {
      kind: "surface";
      sector: number;
      /** The shader table entry drawing it. */
      shaderId: number;
      /** The sector's instanced mesh it is one place of, or none for its baked geometry. */
      mesh: number | null;
      /** Which place of the mesh it is, or none for the baked geometry. */
      place: number | null;
      /** Whether it is a clump of trees drawn as its impostor. */
      isImpostor: boolean;
      point: [number | null, number | null, number | null];
    }
  /** An object the level's spawn places. */
  | {
      kind: "spawn";
      /** Its index among the level's spawned objects. */
      object: number;
      point: [number | null, number | null, number | null];
    };

/**
 * What a viewport's level could not draw the way the level asked: drawables left out of the sectors resident, sectors
 * that could not be read, and spawned models that could not be.
 */
export type RenderLevelProblems = {
  skipped: Array<RenderSectorSkip>;
  sectors: Array<RenderLoadFailure>;
  models: Array<RenderLoadFailure>;
};

/**
 * How a game's console scales the weather's light (`r2_sun_lumscale`, `r2_sun_lumscale_hemi`,
 * `r2_sun_lumscale_amb`): the sun's colour, and the hemisphere and the ambient the deferred frame is combined with.
 */
export type RenderLightScales = {
  sun: number | null;
  hemi: number | null;
  ambient: number | null;
};

/** How a shadowed local light's map is compared. */
export enum ERenderLightShadowFilter {
  /** `shadow_hw`: four bilinear comparisons 0.6 of a texel off the point, at `r2_ls_depth_bias` -0.0003, as vanilla. */
  ENGINE = "engine",
  /** Anomaly's `shadow_pcss`: a blocker search, then a penumbra of twelve comparisons, at its -0.001 bias. */
  SOFT = "soft",
}

/** Every `ERenderLightShadowFilter` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderLightShadowFilter = `${ERenderLightShadowFilter}`;

/** What a level's local lights came to on the last frame counted. */
export type RenderLightsReport = {
  /** Lights standing in view and lit. */
  inView: number;
  /** Of them, lights lit with their shadows. */
  shadowed: number;
  /** Lights in view past the most a frame holds, the farthest. */
  excess: number;
  /** Texels of the shadow atlas held, of its whole. */
  atlas: RenderPoolUse;
  /** Clusters of the view more lights reached than one holds, and the lights they left out. */
  fullClusters: number;
  dropped: number;
};

/** The level's local lights: binned into clusters of the view, and accumulated after the sun in one pass. */
export type RenderLightsSettings = {
  isEnabled: boolean;
  /** Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`. */
  isLevelLights: boolean;
  /** Whether a light the engine shadows casts its shadows. */
  isShadowed: boolean;
  shadowFilter: RenderLightShadowFilter;
};

/**
 * Something of a level that could not be read, by what names it, and why: a sector by its index, a spawned model by
 * its visual's name.
 */
export type RenderLoadFailure = {
  name: string;
  reason: string;
};

/**
 * How far a viewport's level has been read and put on the GPU, sent as `RenderViewportEvent::Load` and answered to a
 * caller polling `describe_load`.
 */
export type RenderLoadReport = {
  /** Sectors resident on the GPU. */
  sectors: number;
  /** Sectors the level has. */
  sectorsTotal: number;
  /** Bytes of the sectors resident, packed. */
  bytes: number;
  /** Textures uploaded or given up on. */
  textures: number;
  /** Textures the level samples: its scene's, its lights' projectors and its particles'. */
  texturesTotal: number;
  /**
   * Whether everything is resident, so the level draws as it will: every sector, the spawn, grass, lights and
   * particles read, and every texture settled.
   */
  isReady: boolean;
};

/**
 * What decides how much of a level's static geometry draws at a distance: the engine's screen area thresholds, each in
 * pixels of a 90 degree lens before `r__geometry_lod` scales them.
 */
export type RenderLodSettings = {
  /** Whether distant trees are drawn as their impostors. */
  isImpostors: boolean;
  /** `r__geometry_lod`: every screen area threshold scales with it. */
  geometryLod: number | null;
  /** `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second. */
  ssaA: number | null;
  ssaB: number | null;
  /** `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn. */
  ssaDiscard: number | null;
  /** `r_ssaGLOD_start` and `r_ssaGLOD_end`: a progressive mesh is whole above the first, coarsest below the second. */
  ssaGlodStart: number | null;
  ssaGlodEnd: number | null;
};

/** What the renderer holds on the GPU for what it draws. */
export type RenderMemoryReport = {
  /** Bytes of every texture uploaded, shared by every viewport. */
  textures: number;
  /** Bytes of this viewport's scene buffers: geometry, clusters, places and the rest that grow with what it shows. */
  scene: number;
};

/** How a viewport's skinned models stand: a frame of a motion of theirs, or their bind pose, and the bones collapsed. */
export type RenderModelPose = {
  /** The motion, by its name, or none for the bind pose. */
  motion: string | null;
  /** Which of its frames; one outside it shows the bind pose. */
  frame: number;
  /** Bones collapsed to nothing, by index, each one's descendants among them. */
  hiddenBones: Array<number>;
};

/** Every `kind` the `RenderOverlay` union is told apart by, so a switch or a comparison names one. */
export enum ERenderOverlay {
  /**
   * Line segments in renderer space: three floats a vertex, two vertices a segment, and three floats of colour a
   * vertex.
   */
  LINES = "lines",
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  SUN = "sun",
  /**
   * Points in renderer space, three floats each, drawn as discs of one colour `size` device pixels across wherever
   * they stand.
   */
  POINTS = "points",
  /** Every skinned model's bones as segments, child to parent, following its pose. */
  SKELETON = "skeleton",
}

/** A helper drawn over a viewport's frame, unlit, as the raw colours it names. */
export type RenderOverlay =
  /**
   * Line segments in renderer space: three floats a vertex, two vertices a segment, and three floats of colour a
   * vertex.
   */
  | {
      kind: "lines";
      positions: Array<number | null>;
      colors: Array<number | null>;
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    }
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  | { kind: "sun"; color: [number | null, number | null, number | null]; size: number | null }
  /**
   * Points in renderer space, three floats each, drawn as discs of one colour `size` device pixels across wherever
   * they stand.
   */
  | {
      kind: "points";
      positions: Array<number | null>;
      color: [number | null, number | null, number | null];
      size: number | null;
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    }
  /** Every skinned model's bones as segments, child to parent, following its pose. */
  | {
      kind: "skeleton";
      color: [number | null, number | null, number | null];
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    };

/** What the page shows where it is transparent around its viewports: its colour, and the wash laid over it. */
export type RenderPageBackdrop = {
  color: RenderColor;
  wash: RenderPageWash | null;
};

/** A linear gradient over a box of the page, as CSS `linear-gradient(angle, from, to)` paints it over a colour. */
export type RenderPageWash = {
  /** The box it is painted over, in device pixels of the window's client area. */
  rect: RenderRect;
  /** Degrees clockwise from pointing up, as CSS states a gradient's angle. */
  angle: number | null;
  /** Its first and last colours, sRGB channels and alpha from 0 to 1. */
  from: [number | null, number | null, number | null, number | null];
  to: [number | null, number | null, number | null, number | null];
};

/** What a level's particle systems came to over the span reported. */
export type RenderParticlesReport = {
  /** Effects playing, each of a group's counted. */
  effects: number;
  /** Particles alive in them. */
  particles: number;
  /** Effects that took an update on the last frame counted, drawn or scheduled. */
  simulated: number;
  /** Effects drawn on the last frame counted. */
  drawn: number;
  /** Mean milliseconds a frame spent stepping them, on the render thread's workers. */
  simulationTime: number | null;
};

/** What one pass of a viewport's frames cost on the GPU. */
export type RenderPassCost = {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds over the report's span. */
  gpuTime: number | null;
};

/** How much of a pool of records a frame used: entries held, and entries it has room for before it grows. */
export type RenderPoolUse = {
  used: number;
  capacity: number;
};

/** A rectangle of a window's client area, in device pixels from its top left corner. */
export type RenderRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

/**
 * How much smaller than the viewport the scene is drawn and then upscaled: FSR's quality modes, by the ratio of the
 * viewport's side to the drawing's.
 */
export enum ERenderScale {
  NATIVE = "native",
  /** 1.5: two thirds of each side. */
  QUALITY = "quality",
  /** 1.7. */
  BALANCED = "balanced",
  /** 2: half of each side. */
  PERFORMANCE = "performance",
}

/** Every `ERenderScale` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderScale = `${ERenderScale}`;

/** A drawable of a level's sector the packer left out, and why. */
export type RenderSectorSkip = {
  sector: number;
  skip: SectorSkip;
};

/**
 * What of a viewport's level is selected, and the colour it is marked in: outlined where it is drawn, and a spawned
 * object boxed too.
 */
export type RenderSelection = {
  target: RenderSelectionTarget;
  /** sRGB from 0 to 1. */
  color: [number | null, number | null, number | null];
};

/** Every `kind` the `RenderSelectionTarget` union is told apart by, so a switch or a comparison names one. */
export enum ERenderSelectionTarget {
  /** An object the level's spawn places, by its index among them. */
  SPAWN = "spawn",
  /**
   * A surface the level compiled: one place of a sector's instanced mesh, or the sector's baked geometry of one shader
   * table entry.
   */
  SURFACE = "surface",
}

/** One thing of a level a selection names, as a pick names it. */
export type RenderSelectionTarget =
  /** An object the level's spawn places, by its index among them. */
  | { kind: "spawn"; object: number }
  /**
   * A surface the level compiled: one place of a sector's instanced mesh, or the sector's baked geometry of one shader
   * table entry.
   */
  | { kind: "surface"; sector: number; shaderId: number; mesh: number | null; place: number | null };

/** What every viewport of the renderer draws with. */
export type RenderSettings = {
  frameRate: RenderFrameRate;
  /** Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes. */
  isGpuTimed: boolean;
};

/**
 * The sun's shadow: cascades of maps, each a square of the level seen from the sun, drawn through the static draws
 * and sampled by the sun's light. The engine's are three, 20, 40 and 160 metres across, at 2048 texels
 * (`render_phase_sun.cpp`, `r2_smap_size`).
 */
export type RenderShadowSettings = {
  isEnabled: boolean;
  /** Each cascade's width in metres, nearest first; as many cascades as widths, at most four. */
  cascades: Array<number | null>;
  /** Texels each cascade's map is across. */
  resolution: number;
  /** Texels the filter reaches from the one sampled, each way: zero for one comparison, one for a three by three. */
  filter: number;
  /** How far a point is moved along its normal before it is compared, in texels of its cascade. */
  bias: number | null;
  /** Metres towards the sun past a cascade that its casters may stand. */
  reach: number | null;
  /** How far in from a cascade's edge, as a share of its width, the next cascade is mixed in. */
  blend: number | null;
  /** Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not. */
  isStaggered: boolean;
};

/** How full a level's static draws' pools are, and what the camera's cull kept and occlusion hid. */
export type RenderStaticReport = {
  /** Slots, a draw each. */
  slots: RenderPoolUse;
  /** Places static draws stand in. */
  places: RenderPoolUse;
  /** Rows the instance cull tests, a spawned model's place each. */
  rows: RenderPoolUse;
  /** Impostors of clumps of trees. */
  lods: RenderPoolUse;
  /** Clusters static draws are made of. */
  clusters: RenderPoolUse;
  /** Entries the camera's visible list held, of the room it has. */
  surfaceList: RenderPoolUse;
  /** Indirect draws the camera's static batches issue a frame. */
  commands: number;
  /** Clusters the camera kept, and their triangles. */
  keptClusters: number;
  keptTriangles: number;
  /** Clusters the frustum kept and the depth hid, and their triangles. */
  occludedClusters: number;
  occludedTriangles: number;
};

/** How the sun's light shafts are drawn, beside the view's switch for them. */
export type RenderSunShafts = {
  quality: RenderSunShaftsQuality;
  /** `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is. */
  minimum: number | null;
};

/**
 * How finely the sun's light shafts step along a ray: the engines' `r2_sun_shafts` and `r2_sunshafts_quality` short
 * of off, which the view's switch is.
 */
export enum ERenderSunShaftsQuality {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
}

/** Every `ERenderSunShaftsQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderSunShaftsQuality = `${ERenderSunShaftsQuality}`;

/** What colour a surface's albedo is drawn with. */
export enum ERenderSurfaceColor {
  /** Its own textures. */
  TEXTURED = "textured",
  /** One grey for every surface, so the shapes and the light read alone. */
  CLAY = "clay",
  /** A tint of its shader table entry, which tells neighbouring surfaces apart. */
  SHADER = "shader",
}

/** Every `ERenderSurfaceColor` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderSurfaceColor = `${ERenderSurfaceColor}`;

/** How much geometry one shader table entry of a viewport's level draws, across the sectors resident. */
export type RenderSurfaceGeometry = {
  shaderId: number;
  /** Drawables of the level's visuals naming the entry. */
  drawables: number;
  /** Triangles drawn at whole detail, a mesh's once for each place it stands in. */
  triangles: number;
  /** What its base coordinate covers over every draw together. */
  span: RenderSurfaceSpan | null;
  /** The narrowest range any single draw covers. */
  narrowest: RenderSurfaceSpan | null;
};

/** The range a base texture coordinate covers over some of a level's geometry. */
export type RenderSurfaceSpan = {
  uMin: number | null;
  uMax: number | null;
  vMin: number | null;
  vMax: number | null;
};

/** What became of one texture reference a viewport's scene samples. */
export type RenderTextureReport = {
  reference: string;
  state: RenderTextureState;
};

/** Every `kind` the `RenderTextureState` union is told apart by, so a switch or a comparison names one. */
export enum ERenderTextureState {
  LOADING = "loading",
  LOADED = "loaded",
  /** Its reference resolved to no file. */
  MISSING = "missing",
  /** Its file could not be read or laid out, and why. */
  FAILED = "failed",
}

/** Where one texture a viewport's scene samples stands. */
export type RenderTextureState =
  | { kind: "loading" }
  | {
      kind: "loaded";
      width: number;
      height: number;
      levels: number;
      /** The file's own layout, as the textures explorer names it. */
      layout: string;
      /** Whether it was expanded to eight bits a channel rather than uploaded as stored. */
      isExpanded: boolean;
    }
  /** Its reference resolved to no file. */
  | { kind: "missing" }
  /** Its file could not be read or laid out, and why. */
  | { kind: "failed"; reason: string };

/** What the scene is drawn at, a share of the viewport upscaled to it, and how sharply the upscaled frame is finished. */
export type RenderUpscalingSettings = {
  scale: RenderScale;
  /** RCAS's sharpness while upscaled, from none to its most. */
  sharpening: number | null;
};

/** What one viewport draws its scene with, as its viewer's toolbar sets it. */
export type RenderViewOptions = {
  /** Whether the scene is lit, else shown as its raw albedo. */
  isLit: boolean;
  /** Whether every static surface draws as its triangles' edges. */
  isWireframe: boolean;
  /** What colour surfaces' albedo is drawn with: their textures, clay, or their shader's tint. */
  surfaceColor: RenderSurfaceColor;
  /** Whether bump textures bend the normal. */
  isBumped: boolean;
  /** How far the baked hemisphere darkens the ambient: zero for not at all. */
  hemiStrength: number | null;
  /** Whether what the last frame's depth hides is left undrawn. */
  isOcclusionCulled: boolean;
  /** How much of the static geometry draws at a distance. */
  lod: RenderLodSettings;
  /** An asset viewer's light, in place of the weather's; none for a level. */
  assetLighting: RenderAssetLighting | null;
  /**
   * What shows where nothing was drawn and neither the sky nor the fog is, each channel zero to one; none for the
   * level viewer's own.
   */
  backdrop: [number | null, number | null, number | null] | null;
  /** The backdrop laid out as a checkerboard with a second colour, as behind a picture with alpha; none for a plain one. */
  backdropSquares: RenderBackdropSquares | null;
  /** Times a uv checker repeats over a surface's base coordinate, drawn in place of its textures; zero for none. */
  checker: number | null;
  /** The colour a surface naming no base texture is drawn, each channel zero to one; none for white. */
  plainColor: [number | null, number | null, number | null] | null;
  /** Whether surfaces cut out and blend as their shaders ask, or draw solid. */
  isAlphaVisible: boolean;
  /** Whether the weather's fog hides the distance. */
  isFogged: boolean;
  /** Whether the weather's sky is drawn behind the level, rather than a plain backdrop. */
  isSkyVisible: boolean;
  /** Whether the distance fades into the sky's haze rather than into the sky itself. */
  isSkyHazed: boolean;
  /** Whether the weather's clouds cross the sky. */
  isClouded: boolean;
  /**
   * Whether the sun's lens flares are drawn over the frame, `disable_lens_flare 0`; its sprite and gradient are drawn
   * either way.
   */
  isLensFlared: boolean;
  /**
   * Whether the sun's light shafts are drawn through its shadow, `r2_sun_shafts` (`r2_sunshafts_mode volumetric` on
   * Monolith) at its highest quality.
   */
  isSunShafted: boolean;
  /** How finely they step, and Monolith's floor under their density. */
  sunShafts: RenderSunShafts;
  /** Whether the weather's rain falls and wets surfaces. */
  isRainy: boolean;
  /** Whether the weather's bolts strike. */
  isThundering: boolean;
  /** Whether the weather's wind sways trees and grass. */
  isWindy: boolean;
  /** Whether the level's wall marks are laid over its surfaces. */
  isWallmarked: boolean;
  /** Whether the level's particle systems play and draw. */
  isParticled: boolean;
  /** Whether its campfires burn, as `CZoneCampfire` starts, rather than smoulder out. */
  isCampfireLit: boolean;
  /** Which groups of the level's spawned objects are drawn. */
  isSpawnedProps: boolean;
  isSpawnedItems: boolean;
  isSpawnedWeapons: boolean;
  isSpawnedLamps: boolean;
  /** Whether the spawned objects a new game releases are drawn too, each with its group. */
  isSpawnedReleased: boolean;
  exposure: RenderExposureSettings;
  shadows: RenderShadowSettings;
  ambientOcclusion: RenderAmbientOcclusionSettings;
  lights: RenderLightsSettings;
  water: RenderWaterSettings;
  grass: RenderGrassSettings;
  /** Which picture the viewport shows: its frame, or one of the targets the frame was built from. */
  debugView: RenderDebugView;
  /** How the frame's edges are smoothed. */
  antialiasing: RenderAntialiasing;
  /** What the scene is drawn at, and how its upscaled frame is sharpened. */
  upscaling: RenderUpscalingSettings;
  /**
   * Device pixels the scene is drawn tall before its render scale, or `None` for the viewport's own; one taller than
   * the viewport draws at the viewport's.
   */
  renderHeight: number | null;
  /** How the game's console scales the sun, the hemisphere and the ambient. */
  lightScales: RenderLightScales;
  /** What the finished frame is corrected by. */
  corrections: RenderImageCorrections;
};

/** Every `kind` the `RenderViewportEvent` union is told apart by, so a switch or a comparison names one. */
export enum ERenderViewportEvent {
  /** What the recent frames cost. */
  FRAME = "frame",
  /** Where the camera stands, sent while it moves and once more after it stops. */
  CAMERA = "camera",
  /** What its frames are drawn with, as the renderer resolved what it was asked, sent as it changes. */
  APPLIED = "applied",
  /** How far its scene has loaded, sent as it changes. */
  LOAD = "load",
  /** Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays. */
  WEATHER = "weather",
  /** The renderer cannot draw this viewport, and why. */
  FAILURE = "failure",
}

/** What a viewport tells its page. */
export type RenderViewportEvent =
  /** What the recent frames cost. */
  | { kind: "frame"; report: RenderFrameReport }
  /** Where the camera stands, sent while it moves and once more after it stops. */
  | { kind: "camera"; pose: RenderCameraPose }
  /** What its frames are drawn with, as the renderer resolved what it was asked, sent as it changes. */
  | { kind: "applied"; report: RenderAppliedReport }
  /** How far its scene has loaded, sent as it changes. */
  | { kind: "load"; report: RenderLoadReport }
  /** Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays. */
  | { kind: "weather"; report: RenderWeatherReport | null }
  /** The renderer cannot draw this viewport, and why. */
  | { kind: "failure"; message: string };

/** One viewport of the renderer, numbered by the renderer as it is attached. */
export type RenderViewportId = number;

/** Where a viewport sits in its window and what the page shows around it. */
export type RenderViewportLayout = {
  /** The viewport element's rectangle, in device pixels of the window's client area. */
  rect: RenderRect;
  /** Device pixels per CSS pixel, the unit input coordinates are given in. */
  scale: number | null;
  /**
   * What the page shows where it is transparent, painted under the viewports: around them, and where one has not
   * followed a layout change yet.
   */
  backdrop: RenderPageBackdrop;
};

/**
 * The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
 * and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
 */
export type RenderWaterSettings = {
  /** Off, what lies under the water shows. */
  isEnabled: boolean;
  /** `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows. */
  isSoft: boolean;
  /** Whether water writes the distortion it causes, moving what is seen through it. */
  isDistorted: boolean;
  /** How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`. */
  waveHeight: number | null;
  /** How fast they run: `W_POSITION_SHIFT_SPEED`. */
  waveSpeed: number | null;
  /** What the two normal layers' scroll is multiplied by, one as the engine scrolls them. */
  ripple: number | null;
  /** What the sky's reflection is multiplied by, one as the engine mixes it. */
  reflection: number | null;
  /**
   * How far the distortion target moves what is behind it, a share of the screen: `def_distort`, which moves what
   * the distorting particles write as well.
   */
  distortion: number | null;
};

/** How a viewport's weather clock runs. */
export type RenderWeatherControl = {
  /** Game seconds a real second, the engine's time factor. */
  factor: number | null;
  isPaused: boolean;
  /** Whether a vanilla cycle stands the sun astronomically rather than by its keyframes. */
  isDynamicSun: boolean;
};

/** The weather effect a viewport plays over its cycle. */
export type RenderWeatherEffectReport = {
  name: string;
  /** Game seconds until the cycle takes over again. */
  remaining: number | null;
};

/** Every `kind` the `RenderWeatherPlay` union is told apart by, so a switch or a comparison names one. */
export enum ERenderWeatherPlay {
  /** Nothing: the level is lit by noon of `default_clear`, standing still. */
  NONE = "none",
  /** A cycle of the level's game, by name, read through the level's source. */
  CYCLE = "cycle",
  /** One keyframe set by hand, played as a cycle of one: its sun stands by its own angles on either engine. */
  KEYFRAME = "keyframe",
}

/** What a level viewport's weather plays. */
export type RenderWeatherPlay =
  /** Nothing: the level is lit by noon of `default_clear`, standing still. */
  | { kind: "none" }
  /** A cycle of the level's game, by name, read through the level's source. */
  | { kind: "cycle"; name: string }
  /** One keyframe set by hand, played as a cycle of one: its sun stands by its own angles on either engine. */
  | { kind: "keyframe"; keyframe: WeatherDescriptor };

/** Where a viewport's weather stands. */
export type RenderWeatherReport = {
  /** Seconds since midnight. */
  time: number | null;
  /** The times of the two keyframes blended between, the effect's own while one plays. */
  between: [number | null, number | null];
  /** How far from the first to the second. */
  weight: number | null;
  /** The effect playing, or none. */
  effect: RenderWeatherEffectReport | null;
  /** How many of the level's modifiers reach the camera. */
  modifiers: number;
  /** What is mixed now as one keyframe, without the modifiers: what a keyframe set by hand starts from. */
  current: WeatherDescriptor;
};

/** How a weather handed to a viewport takes over from what it shows. */
export enum ERenderWeatherTransition {
  /** At once, as the first weather a level shows does. */
  CUT = "cut",
  /** Briefly, as an edit of a keyframe set by hand does. */
  EASE = "ease",
  /** Slowly, as another cycle chosen does. */
  FADE = "fade",
}

/** Every `ERenderWeatherTransition` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderWeatherTransition = `${ERenderWeatherTransition}`;
