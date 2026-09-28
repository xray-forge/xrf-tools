import { uniform } from "three/tsl";
import { PerspectiveCamera, UniformNode } from "three/webgpu";

import { IRendererAmbientOcclusionSettings } from "#/contract/renderer-ambient-occlusion-settings";

/** XeGTAO's `FinalValuePower`, the curve a strength of one gives. */
export const AMBIENT_OCCLUSION_FINAL_POWER: number = 2.2;

/** XeGTAO's `RadiusMultiplier`: what the radius asked is widened by, against screen space's own bias to less. */
export const AMBIENT_OCCLUSION_RADIUS_MULTIPLIER: number = 1.457;

/** The share of the search target's height a horizon is searched across at most, however near the point. */
const AMBIENT_OCCLUSION_MAX_REACH: number = 0.25;

/**
 * What the occlusion's search reads: the settings, and how large a metre is on its target.
 */
export class AmbientOcclusionUniforms {
  /** Metres around a point that what stands there occludes it from: XeGTAO's `EffectRadius * RadiusMultiplier`. */
  public readonly radius: UniformNode<"float", number> = uniform(AMBIENT_OCCLUSION_RADIUS_MULTIPLIER);
  /** What the visibility is raised to: XeGTAO's curve times the strength. */
  public readonly power: UniformNode<"float", number> = uniform(AMBIENT_OCCLUSION_FINAL_POWER);
  /** Metres one pixel of the search target spans at a metre from the camera. */
  public readonly spread: UniformNode<"float", number> = uniform(1);
  /** Pixels of the search target a horizon is searched across at most. */
  public readonly reach: UniformNode<"float", number> = uniform(1);

  /**
   * @param settings - What the occlusion is set to.
   * @param camera - The camera drawing, with its projection current.
   * @param width - The search target's width, in its pixels.
   * @param height - And its height.
   */
  public take(
    settings: IRendererAmbientOcclusionSettings,
    camera: PerspectiveCamera,
    width: number,
    height: number
  ): void {
    this.radius.value = settings.radius * AMBIENT_OCCLUSION_RADIUS_MULTIPLIER;
    this.power.value = AMBIENT_OCCLUSION_FINAL_POWER * settings.strength;
    // Clip x over view x at a metre is the projection's first element, whose inverse is its reciprocal.
    this.spread.value = (2 * camera.projectionMatrixInverse.elements[0]) / Math.max(width, 1);
    this.reach.value = Math.max(height * AMBIENT_OCCLUSION_MAX_REACH, 1);
  }
}
