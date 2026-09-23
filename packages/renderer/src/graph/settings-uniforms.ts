import { uniform } from "three/tsl";

import { IRendererSettings } from "#/contract/renderer-settings";

/**
 * The settings shaders read, as uniforms, so switching one never recompiles a material.
 */
export class SettingsUniforms {
  /** One while light shades the frame, zero for raw albedo. */
  public readonly lit = uniform(1);
  /** One while bump pairs shade the surfaces that bind them. */
  public readonly bumped = uniform(1);
  /** How much of the baked hemisphere occlusion applies. */
  public readonly hemiStrength = uniform(1);

  /**
   * @param settings - The consumer's settings.
   */
  public apply(settings: IRendererSettings): void {
    this.lit.value = settings.isLit ? 1 : 0;
    this.bumped.value = settings.isBumped ? 1 : 0;
    this.hemiStrength.value = settings.hemiStrength;
  }
}
