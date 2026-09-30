import { mix } from "@xrf/math";
import { renderGroup, storage, uniform } from "three/tsl";
import { Node, StorageBufferAttribute, UniformNode, Vector2 } from "three/webgpu";

import { IRendererExposureSettings } from "#/contract/renderer-exposure-settings";

/** Cells a side the frame's luminance is measured over, as `rt_LUM_64` is. */
export const EXPOSURE_CELLS: number = 64;

/**
 * The engine's `s_tonemap`, kept on the GPU where the frame measures it and every tonemap reads it, and the constants
 * that measurement adapts it by: `MiddleGray` of `bloom_luminance_3.ps`.
 */
export class ExposureUniforms {
  /** The adapted scale: one until a frame is measured, and while nothing adapts. */
  public readonly adapted: StorageBufferAttribute = new StorageBufferAttribute(new Float32Array([1]), 1);
  /** Each cell's luminance, the last measurement's. */
  public readonly cells: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(EXPOSURE_CELLS * EXPOSURE_CELLS),
    1
  );
  /** `MiddleGray.x`, `.y` and `.z`: the scale is `x / (luminance * y + z)`. */
  public readonly target: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  public readonly weight: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  public readonly floor: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);
  /** `MiddleGray.w`, `f_luminance_adapt`: how far this frame moves the scale towards what it measured. */
  public readonly blend: UniformNode<"float", number> = uniform(0.5).setGroup(renderGroup);
  /** The measured frame's size in texels. */
  public readonly size: UniformNode<"vec2", Vector2> = uniform(new Vector2(1, 1));
  /** What every tonemap multiplies by: the settings' own scale times the adapted one. */
  public readonly scale: Node<"float">;

  private rate: number = 0.5;

  /**
   * @param tonemapScale - The settings' own scale, which the adapted one multiplies.
   */
  public constructor(tonemapScale: Node<"float">) {
    const adapted: Node<"float"> = storage(this.adapted, "float", 1)
      .toReadOnly()
      .element(0) as unknown as Node<"float">;

    this.scale = tonemapScale.mul(adapted);
  }

  /**
   * `(1, 0, 1)` lerped towards `(middle gray, 1, low luminance)` by the amount, as `phase_luminance` sets `MiddleGray`.
   *
   * @param settings - The exposure's settings.
   */
  public apply(settings: IRendererExposureSettings): void {
    const { amount, middleGray, lowLuminance } = settings;

    this.target.value = mix(1, middleGray, amount);
    this.weight.value = amount;
    this.floor.value = mix(1, lowLuminance, amount);
  }

  /**
   * `f_luminance_adapt = .9 * f + .1 * dt * adaptation`, once a frame.
   *
   * @param delta - Seconds since the frame before.
   * @param adaptation - `r2_tonemap_adaptation`.
   */
  public advance(delta: number, adaptation: number): void {
    this.rate = 0.9 * this.rate + 0.1 * delta * adaptation;
    this.blend.value = this.rate;
  }

  /** Back to the noon answer, for a frame that no longer adapts. */
  public reset(): void {
    (this.adapted.array as Float32Array)[0] = 1;
    this.adapted.needsUpdate = true;
    this.rate = 0.5;
  }
}
