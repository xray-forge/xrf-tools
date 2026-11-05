import { formatDegrees } from "@xrf/format";
import { clamp, IPoint3, toDegrees, toRadians, toRendererFacing, toRendererSpace } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { formatLevelPoint, ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";

/** Where a person asks the camera to stand, as the readout states it: metres, and degrees. */
export interface ILevelGoTo {
  x: number;
  y: number;
  z: number;
  heading: number;
  pitch: number;
}

/** Each value in the order the readout states it, which bare numbers are read in. */
const LEVEL_GO_TO_KEYS: ReadonlyArray<keyof ILevelGoTo> = ["x", "y", "z", "heading", "pitch"];

/** The readout's names, each a number after it: `x -394.9 y 4.6 z 167.7 h 102.2° p -21.7°`. */
const STATED: RegExp = /\b([xyzhp])\s*[:=]?\s*(-?\d+(?:\.\d+)?)/gi;

/** A number in text naming none of the readout's names, as a config writes a position: `-394.9, 4.6, 167.7`. */
const NUMBER: RegExp = /-?\d+(?:\.\d+)?/g;

/** Each of the readout's names, by the value it states. */
const STATED_KEYS: Readonly<Record<string, keyof ILevelGoTo>> = { h: "heading", p: "pitch", x: "x", y: "y", z: "z" };

/** The steepest pitch a go-to faces: straight up or down leaves the camera no heading to keep. */
const MAX_PITCH: number = 89.9;

/**
 * @param camera - Where the camera is, as the readout reads it.
 * @returns The same, in the readout's units.
 */
export function toLevelGoTo(camera: ILevelCamera): ILevelGoTo {
  return {
    heading: toDegrees(camera.heading),
    pitch: toDegrees(camera.pitch),
    x: camera.position.x,
    y: camera.position.y,
    z: camera.position.z,
  };
}

/**
 * @param goTo - A place.
 * @returns It as the camera readout states one, which `parseLevelGoTo` reads back.
 */
export function formatLevelGoTo(goTo: ILevelGoTo): string {
  return `${formatLevelPoint(goTo)} h ${formatDegrees(goTo.heading, 1)} p ${formatDegrees(goTo.pitch, 1)}`;
}

/**
 * @param value - Any values under the readout's names, as a file holds them.
 * @returns Whether they name a whole place: every value a finite number.
 */
export function isLevelGoTo(value: Partial<Record<keyof ILevelGoTo, unknown>>): value is ILevelGoTo {
  return LEVEL_GO_TO_KEYS.every(
    (key: keyof ILevelGoTo) => typeof value[key] === "number" && Number.isFinite(value[key])
  );
}

/**
 * @param goTo - A place.
 * @param decimals - Decimal places each value keeps.
 * @returns The place, each value rounded to them.
 */
export function toRoundedLevelGoTo(goTo: ILevelGoTo, decimals: number): ILevelGoTo {
  const scale: number = 10 ** decimals;

  return {
    heading: Math.round(goTo.heading * scale) / scale,
    pitch: Math.round(goTo.pitch * scale) / scale,
    x: Math.round(goTo.x * scale) / scale,
    y: Math.round(goTo.y * scale) / scale,
    z: Math.round(goTo.z * scale) / scale,
  };
}

/**
 * @param text - Text naming a place as the readout does, a copy of it or part of one; or bare numbers in the readout's
 *   order, as a config writes a position.
 * @param current - What each value is where the text does not state it.
 * @returns The place, or null for text stating none of it.
 */
export function parseLevelGoTo(text: string, current: ILevelGoTo): Nullable<ILevelGoTo> {
  const stated: Partial<ILevelGoTo> = {};

  for (const [, name, value] of text.matchAll(STATED)) {
    stated[STATED_KEYS[name.toLowerCase()]] = Number(value);
  }

  // Text naming none of the readout's values states them in its order.
  if (!Object.keys(stated).length) {
    (text.match(NUMBER) ?? []).slice(0, LEVEL_GO_TO_KEYS.length).forEach((value: string, index: number) => {
      stated[LEVEL_GO_TO_KEYS[index]] = Number(value);
    });
  }

  return Object.keys(stated).length ? { ...current, ...stated } : null;
}

/**
 * @param goTo - Where to stand and which way to face, as the readout states it.
 * @returns Where the camera stands and what it looks at, in renderer space: a metre along the way it faces.
 */
export function toLevelGoToViewpoint(goTo: ILevelGoTo): ILevelViewpoint {
  const position: IPoint3 = toRendererSpace({ x: goTo.x, y: goTo.y, z: goTo.z });
  const pitch: number = clamp(goTo.pitch, -MAX_PITCH, MAX_PITCH);
  const facing: IPoint3 = toRendererFacing({ heading: toRadians(goTo.heading), pitch: toRadians(pitch) });

  return {
    position,
    target: { x: position.x + facing.x, y: position.y + facing.y, z: position.z + facing.z },
  };
}
