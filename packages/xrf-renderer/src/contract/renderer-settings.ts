import { Nullable } from "@xrf/types";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { toRendererFeatureSettings } from "#/contract/renderer-feature-choice";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { IRenderFramePacing } from "#/frame/render-frame-pacing";

/**
 * How the consumer wants frames drawn.
 */
export interface IRendererSettings {
  /** How often a frame may be drawn, and how far frames run ahead of the GPU. */
  pacing: IRenderFramePacing;
  /** What the canvas shows where nothing is drawn, as a hex colour, or null to leave it transparent. */
  backdrop: Nullable<number>;
  /** Which picture reaches the canvas. */
  debugView: ERendererDebugView;
  /** What the tonemap multiplies by first, over whatever the exposure adapts it to. */
  tonemapScale: number;
  /** Whether light shades the frame; unlit, every surface shows its raw albedo, as the file itself reads. */
  isLit: boolean;
  /** Whether bump pairs perturb the normal and supply gloss; off, every surface is shaded flat with `def_gloss`. */
  isBumped: boolean;
  /** Whether the sky is drawn behind the scene and the fog fades into it, as a level's is; off, the backdrop shows. */
  isSkyDrawn: boolean;
  /** Whether surfaces draw as their triangles' edges. */
  isWireframe: boolean;
  /** How much of the baked hemisphere occlusion applies: one as the engine applies it, zero ignoring it. */
  hemiStrength: number;
  /** What every feature is set to, the same for every consumer: a preset and what was changed on top of it. */
  features: IRendererFeatureSettings;
}

/**
 * @param settings - The settings as a consumer sent them.
 * @returns The same settings with their features held to the schema, which is all the renderer trusts past here.
 */
export function toRendererSettings(settings: IRendererSettings): IRendererSettings {
  return { ...settings, features: toRendererFeatureSettings(settings.features) };
}
