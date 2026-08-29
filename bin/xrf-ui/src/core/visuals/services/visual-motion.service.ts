import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";
import { transformError } from "@/core/error/lib";
import { clampMotionFps, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, cancelFlows, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** A posed motion: what the backend said it is, and every frame's bone transforms. */
export interface IPosedMotion {
  bake: VisualMotionBake;
  transforms: Float32Array;
}

/**
 * Playing one of the open visual's motions.
 *
 * The whole motion is fetched once and played locally, because at thirty frames a second a round trip per frame is not
 * playback. What crosses is one transform per bone per frame, which poses the mesh through its skinning and the
 * skeleton overlay through those transforms' translations: one buffer, both surfaces.
 */
@Injectable()
export class VisualMotionService {
  public readonly log: Logger = new Logger(this.constructor.name);

  private ticker: Nullable<ReturnType<typeof setInterval>> = null;

  /**
   * Names the open visual can play, listed on demand.
   *
   * Not fetched with the model: naming them means reading every animation file it references, about fifty milliseconds
   * each, and most models are opened to be looked at rather than played.
   */
  @Observable()
  public motions: Loadable<Array<string>> = createLoadable([]);

  @Observable()
  public posed: Loadable<Nullable<IPosedMotion>> = createLoadable(null);

  @Observable()
  public frame: number = 0;

  @Observable()
  public isPlaying: boolean = false;

  @Observable()
  public isLooping: boolean = true;

  /**
   * Frames a second playback advances at, which starts at the rate the format samples.
   *
   * A viewing aid rather than a property of the motion: the reported duration stays the time the engine would take,
   * which the motion's own speed already scales, and playing it at another rate only changes how long looking at it
   * takes. Slowing a two second animation down is the only way to see what a foot does in three frames.
   */
  @Observable()
  public fps: number = MOTION_SAMPLE_FPS;

  /**
   * @returns Frames the posed motion holds, or zero when nothing is posed.
   */
  @Computed()
  public get frameCount(): number {
    return this.posed.value?.bake.frameCount ?? 0;
  }

  /**
   * @returns Floats one bone occupies in the posed buffer, which the scene needs to index into it.
   *
   * Read off the bake rather than assumed, so the buffer's layout is stated by whoever produced it.
   */
  @Computed()
  public get floatsPerBone(): number {
    return this.posed.value?.bake.floatsPerBone ?? 0;
  }

  /** Stops the ticker when the application goes away, so a hidden viewer is not still animating. */
  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /** Lists what the open visual can play, once. */
  @ExclusiveFlow()
  public *list(): TFlow {
    if (this.motions.isLoading || this.motions.value?.length) {
      return;
    }

    this.motions = this.motions.asLoading();

    try {
      const names: Array<string> = yield* call(visualsCommands.listMotions());

      this.motions = this.motions.asReady(names);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list motions:", transformed);

      this.motions = this.motions.asFailed(transformed, []);
    }
  }

  /**
   * Poses one motion and shows it from its first frame.
   *
   * Playback carries over: a motion picked while another was playing plays, and one picked while another was paused
   * stays paused on frame zero, so comparing two motions frame by frame does not mean pausing each one again. The first
   * pick of a session plays, because picking a motion with nothing posed is asking to see it move.
   *
   * @param name - Motion to pose, as `list` reported it.
   */
  @LatestFlow()
  public *open(name: string): TFlow {
    const resume: boolean = this.isPlaying || !this.posed.value;

    this.stopTicker();

    // Cleared rather than left standing: `play` below is a no-op while this says playback is already running, which
    // would leave the controls claiming to play a motion whose ticker was just stopped.
    this.isPlaying = false;
    // The pose on screen is kept while the next one bakes, rather than dropped. Dropping it put the model back in its
    // bind pose for the length of a read, so switching between two motions meant watching the skeleton snap flat in
    // between - the frame stays put with it, since a frame index means nothing without the bake it counts into.
    this.posed = this.posed.asLoading();

    try {
      const bake: VisualMotionBake = yield* call(visualsCommands.openMotion(name));
      const bytes: ArrayBuffer = yield* call(visualsRawCommands.readMotion(name));

      const expected: number = bake.frameCount * bake.boneCount * bake.floatsPerBone * Float32Array.BYTES_PER_ELEMENT;

      if (bytes.byteLength !== expected) {
        throw new Error(
          `Motion '${name}' returned ${bytes.byteLength} bytes for ${bake.frameCount} frames of ` +
            `${bake.boneCount} bones, which needs ${expected}. The pose and its bytes came from different reads.`
        );
      }

      this.frame = 0;
      this.posed = this.posed.asReady({ bake, transforms: new Float32Array(bytes) });

      if (resume) {
        this.play();
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to pose motion '${name}':`, transformed);

      this.posed = this.posed.asFailed(transformed, null);
    }
  }

  /** Shows one frame, stopping playback so a dragged slider is not fought over. */
  @BoundAction()
  public seek(frame: number): void {
    this.stopTicker();
    this.isPlaying = false;
    this.frame = Math.max(0, Math.min(frame, Math.max(0, this.frameCount - 1)));
  }

  @BoundAction()
  public play(): void {
    if (!this.posed.value || this.isPlaying) {
      return;
    }

    // Play on the last frame restarts, rather than starting a ticker whose first tick has nowhere to go: a motion that
    // ran to its end without looping is parked there, and pressing play on it means play it again.
    if (this.frame >= this.frameCount - 1) {
      this.frame = 0;
    }

    this.isPlaying = true;
    this.startTicker();
  }

  /**
   * Plays at another rate, restarting the ticker so a change is felt without pressing play again.
   *
   * @param fps - Frames a second, clamped to something a `setInterval` can actually keep up with.
   */
  @BoundAction()
  public setFps(fps: number): void {
    this.fps = clampMotionFps(fps);

    if (this.isPlaying) {
      this.startTicker();
    }
  }

  @BoundAction()
  public pause(): void {
    this.stopTicker();
    this.isPlaying = false;
  }

  @BoundAction()
  public toggleLoop(): void {
    this.isLooping = !this.isLooping;
  }

  /**
   * Drops the posed motion and stops playing, which is what leaving or opening another model means.
   *
   * The playback rate survives: it is a preference about looking at motions rather than a property of the one that was
   * open. The list does not, because it named the previous model's motions; whoever shows them lists again for the
   * model that replaced it.
   */
  @BoundAction()
  public clear(): void {
    cancelFlows(this);

    this.stopTicker();
    this.isPlaying = false;
    this.frame = 0;
    this.posed = createLoadable(null);
    this.motions = createLoadable([]);
  }

  /** One frame on, wrapping or stopping at the end depending on the loop toggle. */
  @BoundAction()
  private advance(): void {
    const last: number = this.frameCount - 1;

    if (last <= 0) {
      return;
    }

    if (this.frame >= last) {
      if (!this.isLooping) {
        return this.pause();
      }

      this.frame = 0;

      return;
    }

    this.frame += 1;
  }

  /** (Re)starts the frame ticker at the current rate, replacing any ticker already running. */
  private startTicker(): void {
    this.stopTicker();

    this.ticker = setInterval(() => this.advance(), 1_000 / this.fps);
  }

  private stopTicker(): void {
    if (this.ticker !== null) {
      clearInterval(this.ticker);
      this.ticker = null;
    }
  }
}
