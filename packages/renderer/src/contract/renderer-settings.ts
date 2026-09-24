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
  /** Whether light shades the frame; unlit, every surface shows its raw albedo, as the file itself reads. */
  isLit: boolean;
  /** Whether bump pairs perturb the normal and supply gloss; off, every surface is shaded flat with `def_gloss`. */
  isBumped: boolean;
  /** Whether surfaces draw as their triangles' edges. */
  isWireframe: boolean;
  /** How much of the baked hemisphere occlusion applies: one as the engine applies it, zero ignoring it. */
  hemiStrength: number;
  /** When a clump of trees draws as its impostor instead, as the engine decides it. */
  lod: IRendererLodSettings;
}

/**
 * The engine's switch between a clump of trees and its impostor, on a clump's screen area: its sphere's radius over its
 * squared distance, scaled by `FLOD::lod_factor`, against thresholds that scale with the drawing's size and field of
 * view (`r2_R_calculate.cpp`).
 */
export interface IRendererLodSettings {
  /** Whether impostors draw at all; off, every tree draws in full at every distance. */
  isImpostors: boolean;
  /** `r2_ssa_lod_a`: below it the impostor draws. */
  ssaA: number;
  /** `r2_ssa_lod_b`: above it the trees draw; between the two, both. */
  ssaB: number;
  /** `r__ssa_discard`: below it neither draws. */
  ssaDiscard: number;
  /** `r__geometry_lod`: what the drawing's area is scaled by before the thresholds are taken from it. */
  geometryLod: number;
  /** `r__ssa_glod_start`: above it a progressive mesh draws its whole detail. */
  ssaGlodStart: number;
  /** `r__ssa_glod_end`: below it a progressive mesh draws its coarsest window. */
  ssaGlodEnd: number;
}

/** The engine's own values (`xrRender_console.cpp`). */
export const DEFAULT_RENDERER_LOD_SETTINGS: IRendererLodSettings = {
  geometryLod: 0.75,
  isImpostors: true,
  ssaA: 64,
  ssaB: 48,
  ssaDiscard: 3.5,
  ssaGlodEnd: 64,
  ssaGlodStart: 256,
};
