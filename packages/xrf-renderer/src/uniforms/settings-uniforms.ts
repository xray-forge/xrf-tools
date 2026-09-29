import { renderGroup, uniform } from "three/tsl";
import { UniformNode } from "three/webgpu";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { IRendererFeatureSettings, toRendererUpscale } from "#/contract/renderer-feature-settings";
import { IRendererSettings } from "#/contract/renderer-settings";

/** Frames the counter runs through before it starts again, kept small so the noise stays exact in a float. */
const FRAME_WRAP: number = 1024;

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
  /** One while a temporal resolve averages frames, so a cut-out may be cut against a moving threshold. */
  public readonly stochastic: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** The frames drawn, wrapped, which moves that threshold. */
  public readonly frame: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

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
    this.stochastic.value = isTemporal(settings.features.antialiasing) ? 1 : 0;
  }

  /** Moves the frame on, once a frame. */
  public advance(): void {
    this.frame.value = (this.frame.value + 1) % FRAME_WRAP;
  }
}

/**
 * @param antialiasing - What the frame is smoothed by.
 * @returns Whether it averages frames.
 */
function isTemporal(antialiasing: ERendererAntialiasing): boolean {
  return antialiasing === ERendererAntialiasing.TAA || antialiasing === ERendererAntialiasing.FSR2;
}

/**
 * `log2` of the drawn side over the shown one, and one level finer again under FSR 2 while it upscales, as its
 * integration guide asks.
 *
 * @param features - What the features are set to.
 * @returns Mip levels to sample the surfaces' textures finer by.
 */
export function toTextureBias(features: IRendererFeatureSettings): number {
  const upscale: number = toRendererUpscale(features);
  const bias: number = 0 - Math.log2(upscale);

  // The guide's extra level is for a frame drawn smaller; at the output's size it only sharpens the distance.
  return features.antialiasing === ERendererAntialiasing.FSR2 && upscale > 1 ? bias - 1 : bias;
}
