import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";

import { transformError, XrfApplicationError } from "@/core/error/lib";
import { visualsCommands } from "@/core/ipc/commands/visuals";
import { visualsRawCommands } from "@/core/ipc/commands/visuals-raw";
import { requireSessionId } from "@/core/ipc/session";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { VisualMotionBake } from "@/core/ipc/types/xrf-visual";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Availability of a motion's baked frames.
 */
export enum ESequenceMotionState {
  BAKING = "baking",
  READY = "ready",
  UNAVAILABLE = "unavailable",
}

/** One motion's baked frames, or the reason they could not be loaded. */
export interface ISequenceMotion {
  state: ESequenceMotionState;
  /** Why the motion cannot play, when it cannot. */
  reason: Nullable<string>;
  /** What the backend reported about the completed bake. */
  bake: Nullable<VisualMotionBake>;
  /** Bone transforms for every frame, once read. */
  transforms: Nullable<Float32Array>;
}

/**
 * Owns the sequencer's baked motions and serializes their native reads.
 * Clearing invalidates queued and in-flight results while preserving the queue's ordering.
 */
export class SequenceMotionCache {
  private readonly log: Logger = new Logger(__MODULE_NAME__);

  private baking: Promise<void> = Promise.resolve();
  private generation: number = 0;

  @Observable()
  private motionMap: ReadonlyMap<string, ISequenceMotion> = new Map();

  /**
   * @returns Motion outcomes keyed by name, shared by every clip naming the same motion.
   */
  @Computed()
  public get motions(): ReadonlyMap<string, ISequenceMotion> {
    return this.motionMap;
  }

  /**
   * Creates a cache for the sequencer's visual loader.
   *
   * @param loadService - Supplies the selected model's session when a queued bake starts.
   */
  public constructor(private readonly loadService: VisualLoadService) {
    makeObservable(this);
  }

  /**
   * Queues an uncached motion and records its loading state immediately.
   * Existing outcomes, including failures, are retained until the cache is cleared.
   *
   * @param motion - Motion name reported by the selected model.
   * @returns Completion of a newly queued bake, or immediate completion for an existing entry.
   */
  public async bake(motion: string): Promise<void> {
    if (this.motionMap.has(motion)) {
      return;
    }

    const generation: number = this.generation;

    this.setMotion(motion, { bake: null, reason: null, state: ESequenceMotionState.BAKING, transforms: null });

    this.baking = this.baking.then(() => this.read(motion, generation));

    await this.baking;
  }

  /**
   * Drops cached outcomes and invalidates pending work.
   * New bakes still wait for any native read already in progress.
   */
  @BoundAction()
  public clear(): void {
    this.generation += 1;
    this.motionMap = new Map();
  }

  /**
   * Opens and reads one bake without allowing another queued motion to intervene.
   *
   * @param motion - Motion name to bake.
   * @param generation - Cache generation that requested the bake.
   */
  private async read(motion: string, generation: number): Promise<void> {
    if (generation !== this.generation) {
      return;
    }

    try {
      const sessionId: string = requireSessionId(this.loadService.visual.value?.selected ?? null);
      const snapshot: SessionSnapshot<VisualMotionBake> = await visualsCommands.openMotion(
        sessionId,
        crypto.randomUUID(),
        motion
      );

      const bake: VisualMotionBake = snapshot.value;
      const bytes: ArrayBuffer = await visualsRawCommands.readMotion(sessionId, snapshot.sessionId);
      const expected: number = bake.frameCount * bake.boneCount * bake.floatsPerBone * Float32Array.BYTES_PER_ELEMENT;

      if (bytes.byteLength !== expected) {
        throw new XrfApplicationError(
          `Motion '${motion}' returned ${bytes.byteLength} bytes for ${bake.frameCount} frames of ` +
            `${bake.boneCount} bones, which needs ${expected}. The pose and its bytes came from different reads.`
        );
      }

      if (generation === this.generation) {
        this.setMotion(motion, {
          bake,
          reason: null,
          state: ESequenceMotionState.READY,
          transforms: new Float32Array(bytes),
        });
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to bake motion '${motion}':`, transformed);

      if (generation === this.generation) {
        this.setMotion(motion, {
          bake: null,
          reason: transformed.message,
          state: ESequenceMotionState.UNAVAILABLE,
          transforms: null,
        });
      }
    }
  }

  /**
   * Publishes a complete motion outcome to observers.
   *
   * @param motion - Motion the outcome belongs to.
   * @param state - Loading, ready, or unavailable outcome.
   */
  @BoundAction()
  private setMotion(motion: string, state: ISequenceMotion): void {
    this.motionMap = new Map(this.motionMap).set(motion, state);
  }
}
