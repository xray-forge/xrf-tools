import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { transformError } from "@xrf/errors";
import { formatDuration } from "@xrf/format";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { requireSessionId } from "@xrf/ipc/session";
import { VisualMotionBake } from "@xrf/ipc/types/xrf-visual";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { VisualLoadService } from "@/core/visuals/services/visual-load.service";

/**
 * Availability of a motion's description.
 */
export enum ESequenceMotionState {
  DESCRIBING = "describing",
  READY = "ready",
  UNAVAILABLE = "unavailable",
}

/** One motion's description, or the reason it could not be had. */
export interface ISequenceMotion {
  state: ESequenceMotionState;
  /** Why the motion cannot play, when it cannot. */
  reason: Nullable<string>;
  /** Its frames, bones and length, as a bake of it holds them. */
  bake: Nullable<VisualMotionBake>;
}

/**
 * Owns the sequencer's motion descriptions and serializes their native reads; the renderer bakes a motion as it plays.
 * Clearing invalidates queued and in-flight results while preserving the queue's ordering.
 */
export class SequenceMotionCache {
  private readonly log: Logger = new Logger(__MODULE_NAME__);

  private reading: Promise<void> = Promise.resolve();
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
   * @param loadService - Supplies the selected model's session when a queued read starts.
   */
  public constructor(private readonly loadService: VisualLoadService) {
    makeObservable(this);
  }

  /**
   * Queues an uncached motion and records its loading state immediately.
   * Existing outcomes, including failures, are retained until the cache is cleared.
   *
   * @param motion - Motion name reported by the selected model.
   * @returns Completion of a newly queued read, or immediate completion for an existing entry.
   */
  public async describe(motion: string): Promise<void> {
    if (this.motionMap.has(motion)) {
      return;
    }

    const generation: number = this.generation;

    this.setMotion(motion, { bake: null, reason: null, state: ESequenceMotionState.DESCRIBING });

    this.reading = this.reading.then(() => this.read(motion, generation));

    await this.reading;
  }

  /**
   * Drops cached outcomes and invalidates pending work.
   * New reads still wait for any native read already in progress.
   */
  @BoundAction()
  public clear(): void {
    this.generation += 1;
    this.motionMap = new Map();
  }

  /**
   * Describes one motion without allowing another queued motion to intervene.
   *
   * @param motion - Motion name to describe.
   * @param generation - Cache generation that requested the read.
   */
  private async read(motion: string, generation: number): Promise<void> {
    if (generation !== this.generation) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Describing sequencer motion:", motion);

    try {
      const sessionId: string = requireSessionId(this.loadService.visual.value?.selected ?? null);
      const bake: VisualMotionBake = (await visualsCommands.describeMotion(sessionId, motion)).value;

      if (generation === this.generation) {
        this.setMotion(motion, { bake, reason: null, state: ESequenceMotionState.READY });

        this.log.info(
          "Sequencer motion described:",
          motion,
          bake.frameCount,
          "frames,",
          bake.boneCount,
          "bones, in",
          formatDuration(timer.elapsed())
        );
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to describe motion '${motion}':`, "after", formatDuration(timer.elapsed()), transformed);

      if (generation === this.generation) {
        this.setMotion(motion, {
          bake: null,
          reason: transformed.message,
          state: ESequenceMotionState.UNAVAILABLE,
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
