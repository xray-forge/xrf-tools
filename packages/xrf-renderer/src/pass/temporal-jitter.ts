import { Matrix4, PerspectiveCamera } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { IRendererFrameJitter } from "#/sampling/renderer-frame-jitter";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { MotionUniforms } from "#/uniforms/motion-uniforms";

/** Jitter places a drawn pixel cycles through at the output's size: `ffxFsr2GetJitterPhaseCount`'s base. */
const JITTER_BASE_PHASES: number = 8;

/** The Halton sequence's value at an index in a base, in `[0, 1)`. */
export function toHalton(index: number, base: number): number {
  let fraction: number = 1;
  let result: number = 0;

  for (let rest: number = index; rest > 0; rest = Math.floor(rest / base)) {
    fraction /= base;
    result += fraction * (rest % base);
  }

  return result;
}

/**
 * @param width - The output's width.
 * @param renderWidth - The drawing's width.
 * @returns How many places the jitter cycles through: `ffxFsr2GetJitterPhaseCount`, eight times the squared ratio,
 *   truncated.
 */
export function toTemporalJitterPhases(width: number, renderWidth: number): number {
  return Math.max(1, Math.trunc(JITTER_BASE_PHASES * (width / Math.max(renderWidth, 1)) ** 2));
}

/**
 * @param phase - A frame's place in the cycle.
 * @returns Its offset, in drawn pixels about each pixel's centre, `y` down: `ffxFsr2GetJitterOffset`'s Halton (2, 3).
 */
export function toTemporalJitter(phase: number): readonly [number, number] {
  return [toHalton(phase + 1, 2) - 0.5, toHalton(phase + 1, 3) - 0.5];
}

/**
 * Offsets a perspective projection so the texel at `m` shows the scene at `m + 0.5 + jitter`, through the clip offset a
 * projection carries times `w` in its third column.
 *
 * @param projection - The projection, changed in place.
 * @param x - The jitter across, in pixels.
 * @param y - The jitter down, in pixels.
 * @param width - The drawing's width, in pixels.
 * @param height - And its height.
 */
export function jitterProjection(projection: Matrix4, x: number, y: number, width: number, height: number): void {
  projection.elements[8] += (2 * x) / Math.max(width, 1);
  projection.elements[9] -= (2 * y) / Math.max(height, 1);
}

/**
 * The frame's jitter while a temporal mode resolves: a camera of its own the scene draws with, the view's copied and
 * offset each frame, so the view's camera itself is never jittered.
 */
export class TemporalJitter {
  /** What the scene draws with, one object every frame so three's recordings of it stay put. */
  public readonly camera: PerspectiveCamera = new PerspectiveCamera();

  private readonly motion: MotionUniforms;
  private phase: number = 0;
  private phases: number = JITTER_BASE_PHASES;
  private renderWidth: number = 1;
  private renderHeight: number = 1;

  /**
   * @param motion - The motion uniforms, which the resolves read the jitter from.
   */
  public constructor(motion: MotionUniforms) {
    this.motion = motion;
    adoptRendererConventions(this.camera);
    // Its matrices are copied whole each frame; three must not rebuild them from its position.
    this.camera.matrixWorldAutoUpdate = false;
  }

  /** This frame's place and its cycle. */
  public get state(): IRendererFrameJitter {
    return { offset: toTemporalJitter(this.phase), phases: this.phases };
  }

  /**
   * Starts the cycle again for the frame's size.
   *
   * @param size - The frame's size.
   */
  public resize(size: IRendererFrameSize): void {
    this.phases = toTemporalJitterPhases(size.width, size.renderWidth);
    this.phase = 0;
    this.renderWidth = size.renderWidth;
    this.renderHeight = size.renderHeight;
  }

  /**
   * @param view - The view's camera, its matrices current.
   * @returns The camera the scene draws with this frame: the view's, offset by this frame's place.
   */
  public take(view: PerspectiveCamera): PerspectiveCamera {
    const [x, y] = toTemporalJitter(this.phase);

    this.camera.copy(view, false);
    // `copy` takes the view's flag too, and the matrices it copied are the ones to draw with.
    this.camera.matrixWorldAutoUpdate = false;
    jitterProjection(this.camera.projectionMatrix, x, y, this.renderWidth, this.renderHeight);
    this.camera.projectionMatrixInverse.copy(this.camera.projectionMatrix).invert();
    this.motion.jitter.value.set(x, y);

    return this.camera;
  }

  /** Moves the cycle on, once the frame is resolved. */
  public advance(): void {
    this.phase = (this.phase + 1) % this.phases;
  }

  /** Leaves the samples at the pixels' centres, for frames no longer jittered. */
  public dispose(): void {
    this.motion.jitter.value.set(0, 0);
  }
}
