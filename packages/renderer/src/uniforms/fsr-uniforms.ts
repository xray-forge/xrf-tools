import { uniform } from "three/tsl";
import { PerspectiveCamera, Vector2, Vector4 } from "three/webgpu";

/** `FFX_FSR2_SHADING_CHANGE_MIP_LEVEL`: the luminance mip the locks watch for a change of shading, a 32nd a side. */
export const SHADING_CHANGE_MIP_LEVEL: number = 4;

/**
 * `cbFSR2` as FSR 2's passes read it, set once a frame before they draw.
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
  public readonly frameIndex = uniform(0);

  /**
   * @param camera - The drawing camera.
   * @param render - The scene's size as drawn.
   * @param display - The output's size.
   * @param jitter - The renderer's jitter this frame, in drawn pixels: a texel `m` shows `m + 0.5 + jitter`.
   * @param phases - How many positions the jitter cycles through.
   */
  public follow(
    camera: PerspectiveCamera,
    render: Vector2,
    display: Vector2,
    jitter: readonly [number, number],
    phases: number
  ): void {
    const { near, far } = camera;
    const projection: ReadonlyArray<number> = camera.projectionMatrix.elements;
    const mip: number = 2 << SHADING_CHANGE_MIP_LEVEL;

    this.renderSize.value.copy(render);
    this.displaySize.value.copy(display);
    this.jitter.value.set(-jitter[0], -jitter[1]);
    this.downscale.value.set(render.x / display.x, render.y / display.y);
    // Reversed: `d = n (f - z) / (z (f - n))`, so `z = (n f / (f - n)) / (d + n / (f - n))`.
    this.deviceToView.value.set(
      -near / (far - near),
      (near * far) / (far - near),
      1 / projection[0],
      1 / projection[5]
    );
    this.lumaMipSize.value.set(Math.max(1, Math.floor(render.x / mip)), Math.max(1, Math.floor(render.y / mip)));
    this.jitterPhaseCount.value = phases;
  }
}
