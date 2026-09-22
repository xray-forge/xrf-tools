import { Nullable } from "@xrf/types";

import { TFrameRateLimit } from "#/frame/render-frame-limit";

/**
 * Which picture reaches the canvas: the frame, or one of the targets it was built from.
 */
export enum ERendererDebugView {
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
  /** View depth, near dark and far light. */
  DEPTH = "depth",
  /** What the lights accumulated: diffuse in colour. */
  LIGHT = "light",
}

/**
 * How the consumer wants frames drawn.
 */
export interface IRendererSettings {
  /** How often a frame may be drawn. */
  frameRateLimit: TFrameRateLimit;
  /** What the canvas shows where nothing is drawn, as a hex colour, or null to leave it transparent. */
  backdrop: Nullable<number>;
  /** Which picture reaches the canvas. */
  debugView: ERendererDebugView;
  /** What the tonemap multiplies by first. The engine adapts it to the scene; one is its noon answer. */
  tonemapScale: number;
}
