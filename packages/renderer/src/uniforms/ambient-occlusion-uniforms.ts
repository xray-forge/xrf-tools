import { uniform } from "three/tsl";
import { PerspectiveCamera } from "three/webgpu";

import { IRendererAmbientOcclusionSettings } from "#/contract/renderer-features";

/** XeGTAO's `FinalValuePower`, the curve a strength of one gives. */
export const AMBIENT_OCCLUSION_FINAL_POWER: number = 2.2;

/** The share of the search target's height a horizon is searched across at most, however near the point. */
const AMBIENT_OCCLUSION_MAX_REACH: number = 0.25;

/**
 * What the occlusion's search reads: the settings, and how large a metre is on its target.
 */
export class AmbientOcclusionUniforms {
  /** Metres around a point that what stands there occludes it from. */
  public readonly radius = uniform(1);
  /** What the visibility is raised to: XeGTAO's curve times the strength. */
  public readonly power = uniform(AMBIENT_OCCLUSION_FINAL_POWER);
  /** Metres one pixel of the search target spans at a metre from the camera. */
  public readonly spread = uniform(1);
  /** Pixels of the search target a horizon is searched across at most. */
  public readonly reach = uniform(1);

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
    this.radius.value = settings.radius;
    this.power.value = AMBIENT_OCCLUSION_FINAL_POWER * settings.strength;
    // Clip x over view x at a metre is the projection's first element, whose inverse is its reciprocal.
    this.spread.value = (2 * camera.projectionMatrixInverse.elements[0]) / Math.max(width, 1);
    this.reach.value = Math.max(height * AMBIENT_OCCLUSION_MAX_REACH, 1);
  }
}
