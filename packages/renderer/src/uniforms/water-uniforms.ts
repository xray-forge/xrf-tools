import { renderGroup, texture, uniform } from "three/tsl";
import { DataTexture, FloatType, RedFormat, TextureNode } from "three/webgpu";

import { DEFAULT_RENDERER_WATER_SETTINGS, IRendererWaterSettings } from "#/contract/renderer-features";
import { IRendererLighting } from "#/contract/renderer-lighting";

/**
 * What the water's shaders read: the engine's `timers` and `water_intensity`, and the settings standing
 * in for the constants of `shared/waterconfig.h`.
 */
export class WaterUniforms {
  /** `timers.x`: seconds the renderer has been running. */
  public readonly time = uniform(0).setGroup(renderGroup);
  public readonly waveHeight = uniform(DEFAULT_RENDERER_WATER_SETTINGS.waveHeight).setGroup(renderGroup);
  public readonly waveSpeed = uniform(DEFAULT_RENDERER_WATER_SETTINGS.waveSpeed).setGroup(renderGroup);
  public readonly ripple = uniform(DEFAULT_RENDERER_WATER_SETTINGS.ripple).setGroup(renderGroup);
  public readonly reflection = uniform(DEFAULT_RENDERER_WATER_SETTINGS.reflection).setGroup(renderGroup);
  /** `def_distort`: how far the distortion moves what is behind it, a share of the screen. */
  public readonly distortion = uniform(DEFAULT_RENDERER_WATER_SETTINGS.distortion).setGroup(renderGroup);
  /** One while soft water reads the depth behind it, zero for the engine without `r2_soft_water`. */
  public readonly soft = uniform(1).setGroup(renderGroup);
  /** `water_intensity`. */
  public readonly intensity = uniform(1).setGroup(renderGroup);
  /**
   * The depth behind the water, each pixel's distance along the view in metres, the engine's `s_position.z`: the
   * frame's own once the water pass is made, and nothing near until then.
   */
  public readonly depth: TextureNode = texture(createFarDepthTexture());
  /**
   * @param settings - How the water is drawn.
   */
  public apply(settings: IRendererWaterSettings): void {
    this.waveHeight.value = settings.waveHeight;
    this.waveSpeed.value = settings.waveSpeed;
    this.ripple.value = settings.ripple;
    this.reflection.value = settings.reflection;
    this.distortion.value = settings.distortion;
    this.soft.value = settings.isSoft ? 1 : 0;
  }

  /**
   * @param lighting - How the scene is lit, which names the water's intensity.
   */
  public take(lighting: Pick<IRendererLighting, "waterIntensity">): void {
    this.intensity.value = lighting.waterIntensity;
  }

  /**
   * @param time - Seconds the renderer has been running, the engine's `fTimeGlobal`.
   */
  public update(time: number): void {
    this.time.value = time;
  }
}

/** A texel of a depth farther than any water is deep, in the layout the frame's depth behind the water has. */
function createFarDepthTexture(): DataTexture {
  const far: DataTexture = new DataTexture(new Float32Array([1e6]), 1, 1, RedFormat, FloatType);

  far.needsUpdate = true;

  return far;
}
