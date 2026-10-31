// Auto-generated rust bindings. Do not edit it manually.

import { WeatherDescriptor } from "#/types/xrf-environment";
import { RenderLoadFailure } from "#/types/xrf-renderer";
import { SectorSkip } from "#/types/xrf-visual";

/** The weather's ambient effect playing near a viewport's camera. */
export type WorldAmbientEffectReport = {
  /** Its `effects.ltx` section. */
  name: string;
  /** The `particles.xr` effect or group it plays. */
  particles: string;
  /** Real time its life has left, zero while what it emitted dies out. */
  remaining: number;
};

/** Where the weather's ambient effects near a viewport's camera stand. */
export type WorldAmbientReport = {
  /** The effect playing, or none. */
  effect: WorldAmbientEffectReport | null;
  /** Whether the camera stands indoors, where none starts. */
  isIndoors: boolean;
  /** Real time until the next may start, zero once it may. */
  wait: number;
  /** How many have started since the level opened. */
  played: number;
};

/** Every `kind` the `WorldCamera` union is told apart by, so a switch or a comparison names one. */
export enum EWorldCamera {
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
export type WorldCamera =
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

/** Every `kind` the `WorldCameraCommand` union is told apart by, so a switch or a comparison names one. */
export enum EWorldCameraCommand {
  /** Back to where the camera started. */
  RESET = "reset",
  /** Towards the target or away from it, by a multiplier on the distance: above one moves away. */
  DOLLY = "dolly",
}

/** What a consumer can ask of the camera it described. */
export type WorldCameraCommand =
  /** Back to where the camera started. */
  | { kind: "reset" }
  /** Towards the target or away from it, by a multiplier on the distance: above one moves away. */
  | { kind: "dolly"; step: number | null };

/** Where a camera is and the point it looks at. */
export type WorldCameraPose = {
  position: [number | null, number | null, number | null];
  target: [number | null, number | null, number | null];
};

/** Why a drag over a viewport ended. */
export enum EWorldDragEndReason {
  /** The button was let go. */
  RELEASED = "released",
  /** Something took the drag away: Escape, another window, a lost pointer; an edit drag reverts on it. */
  CANCELLED = "cancelled",
}

/** Every `EWorldDragEndReason` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type WorldDragEndReason = `${EWorldDragEndReason}`;

/** Every `kind` the `WorldInputEvent` union is told apart by, so a switch or a comparison names one. */
export enum EWorldInputEvent {
  /** A press travelled far enough to be a drag; `button` is the `PointerEvent.button` pressed, `0` the main one. */
  DRAG_START = "dragStart",
  /** How far the drag moved since its last move: CSS pixels, or the mouse's own counts while the pointer is locked. */
  DRAG_MOVE = "dragMove",
  /** The drag is over. */
  DRAG_END = "dragEnd",
  /** Where the pointer rests while nothing drags, in CSS pixels from the viewport element's top left corner. */
  HOVER = "hover",
  /** The wheel turned, as `WheelEvent` measures it. */
  WHEEL = "wheel",
  /** A key went down; `code` is `KeyboardEvent.code`, the key's place, so `W` is `W` on azerty too. */
  KEY_DOWN = "keyDown",
  /** A key went up. */
  KEY_UP = "keyUp",
  /** The secondary button asked for a menu, which the page withholds. */
  CONTEXT_MENU = "contextMenu",
  /** Nothing heard held is held now: the viewport lost focus, or lost the pointer lock and the key-ups with it. */
  RELEASE_ALL = "releaseAll",
}

/** What the person does over a viewport: the drags its page told from clicks, the hover, the wheel and the keys. */
export type WorldInputEvent =
  /** A press travelled far enough to be a drag; `button` is the `PointerEvent.button` pressed, `0` the main one. */
  | { kind: "dragStart"; button: number; modifiers: WorldInputModifiers }
  /** How far the drag moved since its last move: CSS pixels, or the mouse's own counts while the pointer is locked. */
  | { kind: "dragMove"; dx: number | null; dy: number | null }
  /** The drag is over. */
  | { kind: "dragEnd"; reason: WorldDragEndReason }
  /** Where the pointer rests while nothing drags, in CSS pixels from the viewport element's top left corner. */
  | { kind: "hover"; x: number | null; y: number | null }
  /** The wheel turned, as `WheelEvent` measures it. */
  | { kind: "wheel"; deltaX: number | null; deltaY: number | null; deltaMode: number }
  /** A key went down; `code` is `KeyboardEvent.code`, the key's place, so `W` is `W` on azerty too. */
  | { kind: "keyDown"; code: string }
  /** A key went up. */
  | { kind: "keyUp"; code: string }
  /** The secondary button asked for a menu, which the page withholds. */
  | { kind: "contextMenu" }
  /** Nothing heard held is held now: the viewport lost focus, or lost the pointer lock and the key-ups with it. */
  | { kind: "releaseAll" };

/** The modifier keys held as a drag's press went down. */
export type WorldInputModifiers = {
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
};

/**
 * What a viewport's level could not draw the way the level asked: drawables left out of the sectors resident, sectors
 * that could not be read, and spawned models that could not be.
 */
export type WorldLevelProblems = {
  skipped: Array<WorldSectorSkip>;
  sectors: Array<RenderLoadFailure>;
  models: Array<RenderLoadFailure>;
};

/** How a viewport's skinned models stand: a frame of a motion of theirs, or their bind pose, and the bones collapsed. */
export type WorldModelPose = {
  /** The motion, by its name, or none for the bind pose. */
  motion: string | null;
  /** Which of its frames; one outside it shows the bind pose. */
  frame: number;
  /** Bones collapsed to nothing, by index, each one's descendants among them. */
  hiddenBones: Array<number>;
};

/** A drawable of a level's sector the packer left out, and why. */
export type WorldSectorSkip = {
  sector: number;
  skip: SectorSkip;
};

/** How much geometry one shader table entry of a viewport's level draws, across the sectors resident. */
export type WorldSurfaceGeometry = {
  shaderId: number;
  /** Drawables of the level's visuals naming the entry. */
  drawables: number;
  /** Triangles drawn at whole detail, a mesh's once for each place it stands in. */
  triangles: number;
  /** What its base coordinate covers over every draw together. */
  span: WorldSurfaceSpan | null;
  /** The narrowest range any single draw covers. */
  narrowest: WorldSurfaceSpan | null;
};

/** The range a base texture coordinate covers over some of a level's geometry. */
export type WorldSurfaceSpan = {
  uMin: number | null;
  uMax: number | null;
  vMin: number | null;
  vMax: number | null;
};

/**
 * What of the level's world plays, rather than what is drawn of it: the weather's rain, bolts and wind, the campfires,
 * the ambient effects, and which groups of the spawned objects stream in.
 */
export type WorldToggles = {
  /** Whether the weather's rain falls and wets surfaces. */
  isRainy: boolean;
  /** Whether the weather's bolts strike. */
  isThundering: boolean;
  /** Whether the weather's wind sways trees and grass. */
  isWindy: boolean;
  /** Whether its campfires burn, as `CZoneCampfire` starts, rather than smoulder out. */
  isCampfireLit: boolean;
  /** Whether the weather's ambient effects play near the camera and bring their wind. */
  isAmbientEffectPlayed: boolean;
  /** Which groups of the level's spawned objects are drawn. */
  isSpawnedProps: boolean;
  isSpawnedItems: boolean;
  isSpawnedWeapons: boolean;
  isSpawnedLamps: boolean;
  /** Whether the spawned objects a new game releases are drawn too, each with its group. */
  isSpawnedReleased: boolean;
  /**
   * The spawned objects left out whatever their group, by their index among the level's spawned objects, ascending:
   * those the viewer hides one by one.
   */
  hiddenSpawnObjects: Array<number>;
};

/** Every `kind` the `WorldViewportEvent` union is told apart by, so a switch or a comparison names one. */
export enum EWorldViewportEvent {
  /** Where the camera stands, sent while it moves and once more after it stops. */
  CAMERA = "camera",
  /** Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays. */
  WEATHER = "weather",
}

/** What a viewport's world tells its page. */
export type WorldViewportEvent =
  /** Where the camera stands, sent while it moves and once more after it stops. */
  | { kind: "camera"; pose: WorldCameraPose }
  /** Where its weather stands, sent as it changes, a few times a second at most; none while nothing plays. */
  | { kind: "weather"; report: WorldWeatherReport | null };

/** How a viewport's weather clock runs. */
export type WorldWeatherControl = {
  /** Game seconds a real second, the engine's time factor. */
  factor: number | null;
  isPaused: boolean;
  /** Whether a vanilla cycle stands the sun astronomically rather than by its keyframes. */
  isDynamicSun: boolean;
};

/** The weather effect a viewport plays over its cycle. */
export type WorldWeatherEffectReport = {
  name: string;
  /** Game seconds until the cycle takes over again. */
  remaining: number | null;
};

/** Every `kind` the `WorldWeatherPlay` union is told apart by, so a switch or a comparison names one. */
export enum EWorldWeatherPlay {
  /** Nothing: the level is lit by noon of `default_clear`, standing still. */
  NONE = "none",
  /** A cycle of the level's game, by name, read through the level's source. */
  CYCLE = "cycle",
  /** One keyframe set by hand, played as a cycle of one: its sun stands by its own angles on either engine. */
  KEYFRAME = "keyframe",
}

/** What a level viewport's weather plays. */
export type WorldWeatherPlay =
  /** Nothing: the level is lit by noon of `default_clear`, standing still. */
  | { kind: "none" }
  /** A cycle of the level's game, by name, read through the level's source. */
  | { kind: "cycle"; name: string }
  /** One keyframe set by hand, played as a cycle of one: its sun stands by its own angles on either engine. */
  | { kind: "keyframe"; keyframe: WeatherDescriptor };

/** Where a viewport's weather stands. */
export type WorldWeatherReport = {
  /** Seconds since midnight. */
  time: number | null;
  /** The times of the two keyframes blended between, the effect's own while one plays. */
  between: [number | null, number | null];
  /** How far from the first to the second. */
  weight: number | null;
  /** The effect playing, or none. */
  effect: WorldWeatherEffectReport | null;
  /** How many of the level's modifiers reach the camera. */
  modifiers: number;
  /** What is mixed now as one keyframe, without the modifiers: what a keyframe set by hand starts from. */
  current: WeatherDescriptor;
  /** The ambient effects near the camera, none until the level's particles are read. */
  ambient: WorldAmbientReport | null;
};

/** How a weather handed to a viewport takes over from what it shows. */
export enum EWorldWeatherTransition {
  /** At once, as the first weather a level shows does. */
  CUT = "cut",
  /** Briefly, as an edit of a keyframe set by hand does. */
  EASE = "ease",
  /** Slowly, as another cycle chosen does. */
  FADE = "fade",
}

/** Every `EWorldWeatherTransition` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type WorldWeatherTransition = `${EWorldWeatherTransition}`;
