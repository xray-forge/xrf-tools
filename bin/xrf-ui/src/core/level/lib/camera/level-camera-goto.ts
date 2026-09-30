import { clamp, toDegrees, toRadians } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { IRenderPoint, toRendererFacing, toRendererSpace } from "@/core/render/lib/scene/render-space";

/** Where a person asks the camera to stand, as the readout states it: metres, and degrees. */
export interface ILevelGoTo {
  x: number;
  y: number;
  z: number;
  heading: number;
  pitch: number;
}

/** Where an empty viewport goes to until it reports where its camera is. */
export const LEVEL_GO_TO_ORIGIN: Readonly<ILevelGoTo> = { heading: 0, pitch: 0, x: 0, y: 0, z: 0 };

/** Each value a person types, under the readout's name for it, in the order the fields show. */
export const LEVEL_GO_TO_FIELDS: ReadonlyArray<readonly [keyof ILevelGoTo, string]> = [
  ["x", "X"],
  ["y", "Y"],
  ["z", "Z"],
  ["heading", "Heading °"],
  ["pitch", "Pitch °"],
];

/** The readout's names, each a number after it: `x -394.9 y 4.6 z 167.7 h 102.2° p -21.7°`. */
const STATED: RegExp = /\b([xyzhp])\s*[:=]?\s*(-?\d+(?:\.\d+)?)/gi;

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
 * @param text - Text naming a place as the readout does, a copy of it or part of one.
 * @param current - What each value is where the text does not name it.
 * @returns The place, or null for text naming none of it.
 */
export function parseLevelGoTo(text: string, current: ILevelGoTo): Nullable<ILevelGoTo> {
  const stated: Partial<Record<string, number>> = {};

  for (const [, name, value] of text.matchAll(STATED)) {
    stated[name.toLowerCase()] = Number(value);
  }

  if (!Object.keys(stated).length) {
    return null;
  }

  return {
    heading: stated.h ?? current.heading,
    pitch: stated.p ?? current.pitch,
    x: stated.x ?? current.x,
    y: stated.y ?? current.y,
    z: stated.z ?? current.z,
  };
}

/**
 * @param goTo - Where to stand and which way to face, as the readout states it.
 * @returns Where the camera stands and what it looks at, in renderer space: a metre along the way it faces.
 */
export function toLevelGoToViewpoint(goTo: ILevelGoTo): ILevelViewpoint {
  const position: IRenderPoint = toRendererSpace({ x: goTo.x, y: goTo.y, z: goTo.z });
  const pitch: number = clamp(goTo.pitch, -MAX_PITCH, MAX_PITCH);
  const facing: IRenderPoint = toRendererFacing({ heading: toRadians(goTo.heading), pitch: toRadians(pitch) });

  return {
    position,
    target: { x: position.x + facing.x, y: position.y + facing.y, z: position.z + facing.z },
  };
}

/**
 * @param goTo - A place.
 * @returns Its values as fields show them, a decimal each.
 */
export function toLevelGoToTexts(goTo: ILevelGoTo): Record<keyof ILevelGoTo, string> {
  return {
    heading: goTo.heading.toFixed(1),
    pitch: goTo.pitch.toFixed(1),
    x: goTo.x.toFixed(1),
    y: goTo.y.toFixed(1),
    z: goTo.z.toFixed(1),
  };
}

/**
 * @param text - A value as typed.
 * @returns Whether it is a number.
 */
export function isLevelGoToText(text: string): boolean {
  return text.trim() !== "" && Number.isFinite(Number(text));
}

/**
 * @param texts - Every value as typed.
 * @returns The place, or null while any of them is not a number.
 */
export function parseLevelGoToTexts(texts: Record<keyof ILevelGoTo, string>): Nullable<ILevelGoTo> {
  if (!LEVEL_GO_TO_FIELDS.every(([key]) => isLevelGoToText(texts[key]))) {
    return null;
  }

  return {
    heading: Number(texts.heading),
    pitch: Number(texts.pitch),
    x: Number(texts.x),
    y: Number(texts.y),
    z: Number(texts.z),
  };
}
