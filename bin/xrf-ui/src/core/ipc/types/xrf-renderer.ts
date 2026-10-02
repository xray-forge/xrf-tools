// Auto-generated rust bindings. Do not edit it manually.

import { WeatherDescriptor } from "@/core/ipc/types/xrf-environment";

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
}

/** Every `ERenderAntialiasing` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAntialiasing = `${ERenderAntialiasing}`;

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
  /** Clusters the last counted frame drew. */
  clusters: number;
  /** Triangles they hold, an instanced one counted for every place it stood. */
  triangles: number;
  /** The graphics API drawn with. */
  backend: string;
  /** The GPU drawn on. */
  adapter: string;
  /** Whether its passes were timed on the GPU over the span. */
  isGpuTimed: boolean;
  /** What each pass cost on the GPU, in frame order; none while untimed. */
  passes: Array<RenderPassCost>;
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

/** The level's local lights: binned into clusters of the view, and accumulated after the sun in one pass. */
export type RenderLightsSettings = {
  isEnabled: boolean;
  /** Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`. */
  isLevelLights: boolean;
  /** Whether a light the engine shadows casts its shadows. */
  isShadowed: boolean;
  shadowFilter: RenderLightShadowFilter;
};

/** How far a viewport's scene has been read and put on the GPU. */
export type RenderLoadReport = {
  /** Sectors resident on the GPU. */
  sectors: number;
  /** Sectors the level has. */
  sectorsTotal: number;
  /** Bytes of the sectors resident, packed. */
  bytes: number;
  /** Textures uploaded or given up on. */
  textures: number;
  /** Textures the scene names. */
  texturesTotal: number;
  /** Whether everything is resident, so the scene draws as it will. */
  isReady: boolean;
};

/** What one pass of a viewport's frames cost on the GPU. */
export type RenderPassCost = {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds over the report's span. */
  gpuTime: number | null;
};

/** How often frames are presented. */
export enum ERenderPresentation {
  /** One frame a refresh of the display. */
  VSYNC = "vsync",
  /** As fast as the frame can be drawn, for measuring. */
  UNCAPPED = "uncapped",
}

/** Every `ERenderPresentation` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderPresentation = `${ERenderPresentation}`;

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

/** What every viewport of the renderer draws with. */
export type RenderSettings = {
  presentation: RenderPresentation;
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
  /** Whether surfaces wear their textures, else their flat colours. */
  isTextured: boolean;
  /** Whether bump textures bend the normal. */
  isBumped: boolean;
  /** How far the baked hemisphere darkens the ambient: zero for not at all. */
  hemiStrength: number | null;
  /** Whether what the last frame's depth hides is left undrawn. */
  isOcclusionCulled: boolean;
  /** Whether distant trees are drawn as their impostors. */
  isImpostors: boolean;
  /** `r__geometry_lod`: every screen area threshold scales with it. */
  geometryLod: number | null;
  /** What the tonemap multiplies by before the exposure's own scale. */
  tonemapScale: number | null;
  /** Whether the weather's fog hides the distance. */
  isFogged: boolean;
  /** Whether the weather's sky is drawn behind the level, rather than a plain backdrop. */
  isSkyVisible: boolean;
  /** Whether the distance fades into the sky's haze rather than into the sky itself. */
  isSkyHazed: boolean;
  /** Whether the weather's clouds cross the sky. */
  isClouded: boolean;
  /** Whether the weather's rain falls and wets surfaces. */
  isRainy: boolean;
  /** Whether the weather's bolts strike. */
  isThundering: boolean;
  /** Whether the weather's wind sways trees and grass. */
  isWindy: boolean;
  /** Whether the level's wall marks are laid over its surfaces. */
  isWallmarked: boolean;
  /** Which groups of the level's spawned objects are drawn. */
  isSpawnedProps: boolean;
  isSpawnedItems: boolean;
  isSpawnedWeapons: boolean;
  isSpawnedLamps: boolean;
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
   * The page's background around the viewport, cleared where the page is transparent but the viewport has not
   * followed a layout change yet.
   */
  clear: RenderColor;
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
  /** Whether water moves what is seen through it, as the engine's distortion target does. */
  isDistorted: boolean;
  /** How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`. */
  waveHeight: number | null;
  /** How fast they run: `W_POSITION_SHIFT_SPEED`. */
  waveSpeed: number | null;
  /** What the two normal layers' scroll is multiplied by, one as the engine scrolls them. */
  ripple: number | null;
  /** What the sky's reflection is multiplied by, one as the engine mixes it. */
  reflection: number | null;
  /** How far the distortion moves what is behind it, a share of the screen: `def_distort`. */
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
