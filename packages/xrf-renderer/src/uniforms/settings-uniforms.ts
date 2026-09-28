import { renderGroup, uniform } from "three/tsl";
import { UniformNode } from "three/webgpu";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { IRendererFeatureSettings, toRendererUpscale } from "#/contract/renderer-feature-settings";
import { IRendererSettings } from "#/contract/renderer-settings";

/**
 * The settings shaders read, as uniforms, so switching one never recompiles a material.
 */
export class SettingsUniforms {
  /** One while light shades the frame, zero for raw albedo. */
  public readonly lit: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** One while surfaces draw their own textures, zero for their flat colours. */
  public readonly textured: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** One while bump pairs shade the surfaces that bind them. */
  public readonly bumped: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** How much of the baked hemisphere occlusion applies. */
  public readonly hemiStrength: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** What the tonemap multiplies by first. */
  public readonly tonemapScale: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** Mip levels every surface texture is sampled finer by, so a scene drawn smaller keeps its shown detail. */
  public readonly textureBias: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /**
   * @param settings - The consumer's settings.
   */
  public apply(settings: IRendererSettings): void {
    this.lit.value = settings.isLit ? 1 : 0;
    this.textured.value = settings.isTextured ? 1 : 0;
    this.bumped.value = settings.isBumped ? 1 : 0;
    this.hemiStrength.value = settings.hemiStrength;
    this.tonemapScale.value = settings.tonemapScale;
    this.textureBias.value = toTextureBias(settings.features);
  }
}

/**
 * `log2` of the drawn side over the shown one, and one level finer again under FSR 2, as its integration guide asks.
 *
 * @param features - What the features are set to.
 * @returns Mip levels to sample the surfaces' textures finer by.
 */
export function toTextureBias(features: IRendererFeatureSettings): number {
  const bias: number = -Math.log2(toRendererUpscale(features));

  return features.antialiasing === ERendererAntialiasing.FSR2 ? bias - 1 : bias;
}
