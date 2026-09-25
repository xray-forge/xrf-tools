import { Nullable } from "@xrf/types";
import { Matrix4, PerspectiveCamera } from "three/webgpu";

import { MotionUniforms } from "#/uniforms/motion-uniforms";

/** Jitter positions a drawn pixel cycles through at the output's size: eight of Halton (2, 3). */
const JITTER_SAMPLES: number = 8;

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
 * @param upscale - The ratio of the output's side to the drawing's.
 * @returns How many jitter positions the frames cycle through: `ffxFsr2GetJitterPhaseCount`, eight times the ratio
 *   squared, so every output pixel a drawn pixel covers is sampled near its centre as often as at the output's size.
 */
export function toTemporalJitterPhases(upscale: number): number {
  return Math.ceil(JITTER_SAMPLES * upscale * upscale - 1e-6);
}

/**
 * @param phase - The frame's place in the cycle.
 * @returns The offset its samples are jittered by, in drawn pixels about the pixel's centre, `y` down.
 */
export function toTemporalJitter(phase: number): readonly [number, number] {
  return [toHalton(phase + 1, 2) - 0.5, toHalton(phase + 1, 3) - 0.5];
}

/**
 * @param output - A side of the output, in pixels.
 * @param upscale - The ratio of the output's side to the drawing's.
 * @returns That side of the scene as drawn.
 */
export function toRenderSize(output: number, upscale: number): number {
  return Math.max(1, Math.round(output / upscale));
}

/**
 * Offsets a perspective projection so the texel at `m` shows the scene at `m + 0.5 + jitter`: the image moves the other
 * way, by as many clip units. A perspective projection carries a clip offset times `w`, which is `-z`, in its third
 * column.
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
 * The camera's jitter for a temporal resolve: a position of the cycle each frame, the projection offset by it before
 * anything draws and given back before the helpers do, and the motion uniforms told where the samples stand.
 */
export class TemporalJitter {
  private readonly motion: MotionUniforms;
  /** The camera's projection as its controller left it, which the helpers draw with. */
  private readonly projection: Matrix4 = new Matrix4();
  private readonly projectionInverse: Matrix4 = new Matrix4();
  private jittered: Nullable<PerspectiveCamera> = null;
  private phase: number = 0;
  private phases: number = JITTER_SAMPLES;
  private width: number = 1;
  private height: number = 1;

  public constructor(motion: MotionUniforms) {
    this.motion = motion;
  }

  /** How many positions the cycle holds. */
  public get phaseCount(): number {
    return this.phases;
  }

  /** This frame's offset, in drawn pixels, `y` down. */
  public get offset(): readonly [number, number] {
    return toTemporalJitter(this.phase);
  }

  /**
   * Starts the cycle again for a drawing's size.
   *
   * @param upscale - The ratio of the output's side to the drawing's.
   * @param width - The drawing's width.
   * @param height - And its height.
   */
  public resize(upscale: number, width: number, height: number): void {
    this.phases = toTemporalJitterPhases(upscale);
    this.phase = 0;
    this.width = width;
    this.height = height;
  }

  /**
   * Offsets the camera's projection by this frame's jitter, before anything reads it.
   *
   * @param camera - The drawing camera, its projection as its controller left it.
   */
  public apply(camera: PerspectiveCamera): void {
    const [x, y] = this.offset;

    this.projection.copy(camera.projectionMatrix);
    this.projectionInverse.copy(camera.projectionMatrixInverse);
    this.jittered = camera;
    this.motion.jitter.value.set(x, y);
    jitterProjection(camera.projectionMatrix, x, y, this.width, this.height);
    camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
  }

  /** Moves the cycle on, once the frame is resolved. */
  public advance(): void {
    this.phase = (this.phase + 1) % this.phases;
  }

  /** Gives the camera back the projection its controller left it, for the helpers and the next frame. */
  public restore(): void {
    if (this.jittered) {
      this.jittered.projectionMatrix.copy(this.projection);
      this.jittered.projectionMatrixInverse.copy(this.projectionInverse);
      this.jittered = null;
    }
  }

  /** Restores the camera and leaves the samples at the pixels' centres, for a frame no longer jittered. */
  public dispose(): void {
    this.restore();
    this.motion.jitter.value.set(0, 0);
  }
}
