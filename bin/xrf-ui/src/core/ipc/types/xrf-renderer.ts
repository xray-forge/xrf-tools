// Auto-generated rust bindings. Do not edit it manually.

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
  /** Clusters the last counted frame drew. */
  clusters: number;
  /** Triangles they hold, an instanced one counted for every place it stood. */
  triangles: number;
  /** The graphics API drawn with. */
  backend: string;
  /** The GPU drawn on. */
  adapter: string;
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

/** What of a level is drawn under a point of a viewport, and where the ray from the eye met it. */
export type RenderLevelHit = {
  sector: number;
  /** The shader table entry drawing it. */
  shaderId: number;
  /** The sector's instanced mesh it is one place of, or none for its baked geometry. */
  mesh: number | null;
  /** Which place of the mesh it is, or none for the baked geometry. */
  place: number | null;
  /** Whether it is a clump of trees drawn as its impostor. */
  isImpostor: boolean;
  /** Where the ray met it, in renderer space. */
  point: [number | null, number | null, number | null];
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

/** What every viewport of the renderer draws with. */
export type RenderSettings = {
  presentation: RenderPresentation;
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

/** What one viewport draws its scene with, as its viewer's toolbar sets it. */
export type RenderViewOptions = {
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
};

/** Every `kind` the `RenderViewportEvent` union is told apart by, so a switch or a comparison names one. */
export enum ERenderViewportEvent {
  /** What the recent frames cost. */
  FRAME = "frame",
  /** Where the camera stands, sent while it moves and once more after it stops. */
  CAMERA = "camera",
  /** How far its scene has loaded, sent as it changes. */
  LOAD = "load",
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
