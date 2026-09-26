import { uniform } from "three/tsl";
import { PerspectiveCamera, Vector2, Vector4 } from "three/webgpu";

import { IRendererFrameJitter } from "#/sampling/renderer-frame-jitter";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";

/** `FFX_FSR2_SHADING_CHANGE_MIP_LEVEL`: the luminance mip the locks watch for a change of shading. */
export const SHADING_CHANGE_MIP_LEVEL: number = 4;

/** Drawn texels a side of the shading change mip spans: `2 << FFX_FSR2_SHADING_CHANGE_MIP_LEVEL`. */
export const SHADING_CHANGE_MIP_DIVISOR: number = 2 << SHADING_CHANGE_MIP_LEVEL;

/**
 * @param side - A side of the drawing, in texels.
 * @returns That side of the shading change mip: `iLumaMipDimensions`.
 */
export function toShadingChangeMipSide(side: number): number {
  return Math.max(1, Math.floor(side / SHADING_CHANGE_MIP_DIVISOR));
}

/**
 * `cbFSR2` as FSR 2's passes read it, taken once a frame before they draw.
 */
export class FsrUniforms {
  public readonly renderSize = uniform(new Vector2(1, 1));
  public readonly displaySize = uniform(new Vector2(1, 1));
  /** In FSR's sense, the renderer's jitter turned: a drawn texel `m` stands at `m + 0.5 - jitter`. */
  public readonly jitter = uniform(new Vector2());
  public readonly downscale = uniform(new Vector2(1, 1));
  public readonly deviceToView = uniform(new Vector4(0, 1, 1, 1));
  public readonly lumaMipSize = uniform(new Vector2(1, 1));
  public readonly jitterPhaseCount = uniform(8);
  /** Zero on the first frame after a reset. */
  public readonly frameIndex = uniform(0);

  /**
   * @param camera - The drawing camera.
   * @param size - The frame's size.
   * @param jitter - This frame's jitter.
   * @param frameIndex - Frames resolved since the history was last reset.
   */
  public take(
    camera: PerspectiveCamera,
    size: IRendererFrameSize,
    jitter: IRendererFrameJitter,
    frameIndex: number
  ): void {
    const { near, far } = camera;
    const projection: ReadonlyArray<number> = camera.projectionMatrix.elements;

    this.renderSize.value.set(size.renderWidth, size.renderHeight);
    this.displaySize.value.set(size.width, size.height);
    this.jitter.value.set(-jitter.offset[0], -jitter.offset[1]);
    this.downscale.value.set(size.renderWidth / size.width, size.renderHeight / size.height);
    // Reversed: `d = n (f - z) / (z (f - n))`, so `z = (n f / (f - n)) / (d + n / (f - n))`.
    this.deviceToView.value.set(
      -near / (far - near),
      (near * far) / (far - near),
      1 / projection[0],
      1 / projection[5]
    );
    this.lumaMipSize.value.set(toShadingChangeMipSide(size.renderWidth), toShadingChangeMipSide(size.renderHeight));
    this.jitterPhaseCount.value = jitter.phases;
    this.frameIndex.value = frameIndex;
  }
}
