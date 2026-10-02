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

/** Every `kind` the `RenderViewportEvent` union is told apart by, so a switch or a comparison names one. */
export enum ERenderViewportEvent {
  /** What the recent frames cost. */
  FRAME = "frame",
  /** Where the camera stands, sent while it moves and once more after it stops. */
  CAMERA = "camera",
  /** The renderer cannot draw this viewport, and why. */
  FAILURE = "failure",
}

/** What a viewport tells its page. */
export type RenderViewportEvent =
  /** What the recent frames cost. */
  | { kind: "frame"; report: RenderFrameReport }
  /** Where the camera stands, sent while it moves and once more after it stops. */
  | { kind: "camera"; pose: RenderCameraPose }
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
