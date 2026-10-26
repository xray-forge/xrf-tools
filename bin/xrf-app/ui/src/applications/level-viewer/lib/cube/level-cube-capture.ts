import {
  RENDER_CUBE_MOST_PIXELS,
  RENDER_CUBE_MOST_SUPERSAMPLING,
  RenderCubeCapture,
} from "@xrf/ipc/types/xrf-renderer";

import { LEVEL_CAPTURE_DRAWN_MOST } from "@/applications/level-viewer/lib/capture/level-capture";

/** Everything a cubemap is captured with, as its dialog holds it. */
export interface ILevelCubeParameters {
  /** Pixels a face takes on a side. */
  size: number;
  /** Times over each side a face is drawn before it is reduced. */
  supersampling: number;
  /** Frames each face settles from where it stands before the one kept. */
  settleFrames: number;
}

/** The face sizes a cubemap is captured at. */
export const LEVEL_CUBE_SIZES: ReadonlyArray<number> = [256, 512, 1024, RENDER_CUBE_MOST_PIXELS];

/** The face size a cubemap is captured at by default. */
export const LEVEL_CUBE_DEFAULT_SIZE: number = 512;

/** The largest face drawn twice over by default: past it a face costs too much to draw so. */
const SUPERSAMPLED_LARGEST: number = 1024;

/** The supersamplings offered, least first. */
export const LEVEL_CUBE_SUPERSAMPLINGS: ReadonlyArray<number> = [1, 2, RENDER_CUBE_MOST_SUPERSAMPLING];

/** Frames a face settles for by default: what the temporal passes take to converge from a fresh start. */
export const LEVEL_CUBE_SETTLE_FRAMES: number = 16;

/** Faces across and down the horizontal cross a cubemap is saved as: four in a row, one above it and one below. */
const CROSS_FACES: Readonly<[number, number]> = [4, 3];

/**
 * @param face - A face's pixels on a side.
 * @returns The pixels across and down of the cross its cubemap is saved as.
 */
export function toLevelCubeCrossSize(face: number): [number, number] {
  return [face * CROSS_FACES[0], face * CROSS_FACES[1]];
}

/**
 * @param width - A cubemap's cross, its pixels across.
 * @returns A face's pixels on a side.
 */
export function toLevelCubeFaceSize(width: number): number {
  return width / CROSS_FACES[0];
}

/**
 * @param size - A face's pixels on a side.
 * @returns The supersampling it is drawn with by default: twice over, but once for the largest faces.
 */
export function toLevelCubeSupersampling(size: number): number {
  return size > SUPERSAMPLED_LARGEST ? 1 : 2;
}

/**
 * @param size - A face's pixels on a side.
 * @param supersampling - Times over it is drawn.
 * @returns Whether every GPU draws the face that many times over.
 */
export function isLevelCubeSupersamplingDrawn(size: number, supersampling: number): boolean {
  return size * supersampling <= LEVEL_CAPTURE_DRAWN_MOST;
}

/**
 * @param parameters - A cubemap's parameters.
 * @returns Whether they ask for something that can be captured: faces every GPU draws.
 */
export function isLevelCubeCapturable(parameters: ILevelCubeParameters): boolean {
  return isLevelCubeSupersamplingDrawn(parameters.size, parameters.supersampling);
}

/**
 * @returns What the dialog opens on: a cubemap of the default face size.
 */
export function toDefaultLevelCubeParameters(): ILevelCubeParameters {
  return {
    size: LEVEL_CUBE_DEFAULT_SIZE,
    supersampling: toLevelCubeSupersampling(LEVEL_CUBE_DEFAULT_SIZE),
    settleFrames: LEVEL_CUBE_SETTLE_FRAMES,
  };
}

/**
 * @param parameters - A cubemap's parameters.
 * @returns What the renderer is asked to capture.
 */
export function toRenderCubeCapture(parameters: ILevelCubeParameters): RenderCubeCapture {
  const { size, supersampling, settleFrames } = parameters;

  return { size, supersampling, settleFrames };
}
