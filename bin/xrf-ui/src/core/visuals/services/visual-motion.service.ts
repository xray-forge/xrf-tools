import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";
import { transformError } from "@/core/error/lib";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** A posed motion: what the backend said it is, and every frame's joint positions. */
export interface IPosedMotion {
  bake: VisualMotionBake;
  joints: Float32Array;
}

/**
 * Playing one of the open visual's motions.
 *
 * The whole motion is fetched once and played locally, because at thirty frames a second a round trip per frame is not
 * playback. What crosses is joint positions rather than bone matrices: the skeleton overlay draws segments between
 * joints, so positions are all it can use.
 */
@Injectable()
export class VisualMotionService {
  /** Frames a second an X-Ray motion samples at, which is what playback has to run at to look right. */
  public static readonly SAMPLE_FPS: number = 30;

  public readonly log: Logger = new Logger(this.constructor.name);

  /** Discards a motion whose bytes arrive after the user picked another. */
  private requestId: number = 0;

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
   * @returns Frames the posed motion holds, or zero when nothing is posed.
   */
  @Computed()
  public get frameCount(): number {
    return this.posed.value?.bake.frameCount ?? 0;
  }

  /**
   * @returns Floats one frame of the joint buffer occupies, which the scene needs to index into it.
   */
  @Computed()
  public get jointStride(): number {
    return (this.posed.value?.bake.boneCount ?? 0) * 3;
  }

  public constructor() {
    makeObservable(this);
  }

  /** Stops the ticker when the application goes away, so a hidden viewer is not still animating. */
  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /** Lists what the open visual can play, once. */
  @BoundAction()
  public async list(): Promise<void> {
    if (this.motions.isLoading || this.motions.value?.length) {
      return;
    }

    runInAction(() => {
      this.motions = this.motions.asLoading();
    });

    try {
      const names: Array<string> = await visualsCommands.listMotions();

      runInAction(() => {
        this.motions = this.motions.asReady(names);
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list motions:", transformed);

      runInAction(() => {
        this.motions = this.motions.asFailed(transformed, []);
      });
    }
  }

  /**
   * Poses one motion and starts it.
   *
   * @param name - Motion to pose, as `list` reported it.
   */
  @BoundAction()
  public async open(name: string): Promise<void> {
    const request: number = runInAction(() => {
      this.requestId += 1;
      this.stopTicker();
      this.posed = this.posed.asLoading();
      this.frame = 0;

      return this.requestId;
    });

    try {
      const bake: VisualMotionBake = await visualsCommands.openMotion(name);
      const bytes: ArrayBuffer = await visualsRawCommands.readMotion(name);

      if (request !== this.requestId) {
        return this.log.info("Discarding a motion already moved past:", name);
      }

      const expected: number = bake.frameCount * bake.boneCount * 3 * Float32Array.BYTES_PER_ELEMENT;

      if (bytes.byteLength !== expected) {
        throw new Error(
          `Motion '${name}' returned ${bytes.byteLength} bytes for ${bake.frameCount} frames of ` +
            `${bake.boneCount} bones, which needs ${expected}. The pose and its bytes came from different reads.`
        );
      }

      runInAction(() => {
        this.posed = this.posed.asReady({ bake, joints: new Float32Array(bytes) });
      });

      this.play();
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to pose motion '${name}':`, transformed);

      runInAction(() => {
        this.posed = this.posed.asFailed(transformed, null);
      });
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
    this.ticker = setInterval(() => this.advance(), 1000 / VisualMotionService.SAMPLE_FPS);
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

  /** Drops the posed motion and stops playing, which is what leaving or opening another model means. */
  @BoundAction()
  public clear(): void {
    this.stopTicker();

    runInAction(() => {
      this.requestId += 1;
      this.isPlaying = false;
      this.frame = 0;
      this.posed = createLoadable(null);
      this.motions = createLoadable([]);
    });
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

  private stopTicker(): void {
    if (this.ticker !== null) {
      clearInterval(this.ticker);
      this.ticker = null;
    }
  }
}
