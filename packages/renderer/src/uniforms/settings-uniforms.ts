import { renderGroup, uniform } from "three/tsl";

import { IRendererSettings } from "#/contract/renderer-settings";

/**
 * The settings shaders read, as uniforms, so switching one never recompiles a material.
 */
export class SettingsUniforms {
  /** One while light shades the frame, zero for raw albedo. */
  public readonly lit = uniform(1).setGroup(renderGroup);
  /** One while bump pairs shade the surfaces that bind them. */
  public readonly bumped = uniform(1).setGroup(renderGroup);
  /** How much of the baked hemisphere occlusion applies. */
  public readonly hemiStrength = uniform(1).setGroup(renderGroup);
  /** What the tonemap multiplies by first. */
  public readonly tonemapScale = uniform(1).setGroup(renderGroup);

  /**
   * @param settings - The consumer's settings.
   */
  public apply(settings: IRendererSettings): void {
    this.lit.value = settings.isLit ? 1 : 0;
    this.bumped.value = settings.isBumped ? 1 : 0;
    this.hemiStrength.value = settings.hemiStrength;
    this.tonemapScale.value = settings.tonemapScale;
  }
}
