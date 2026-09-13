import { JobProgress, ProgressLevel } from "@/core/ipc/types/xrf-job";
import { IJobPhase, IJobProfile } from "@/core/jobs/metrics/jobs-metrics.types";
import { Maybe, Nullable, Optional } from "@/lib/types/general";

/** Matched to the backend's own `RETAINED_JOBS`. */
const RETAINED_PROFILES: number = 20;

/** A profile while it is still being filled in. */
interface IJobProfileState extends IJobProfile {
  /** The phase the last report was in, which is the one the next slice of time belongs to. */
  activePhase: Nullable<IJobPhase>;
  /** What the registry said the run had taken when it last reported. */
  lastDuration: number;
  /** What the innermost level had counted when it last reported. */
  lastCompleted: number;
  /** Milliseconds of reports that have moved nothing, so far. */
  stalledFor: number;
}

/**
 * What this window saw of the runs it watched.
 */
export class JobProfileRecorder {
  /**
   * @param state - Profile being filled in.
   * @returns It as a reader sees it, without the bookkeeping the next report needs.
   */
  private static toProfile(state: IJobProfileState): IJobProfile {
    return {
      id: state.id,
      kind: state.kind,
      isPartial: state.isPartial,
      isFinished: state.isFinished,
      samples: state.samples,
      duration: state.duration,
      phases: state.phases.map((it: IJobPhase) => ({ ...it })),
      peakRate: state.peakRate,
      peakUnit: state.peakUnit,
      longestStall: state.longestStall,
    };
  }

  private readonly profiles: Map<string, IJobProfileState> = new Map();

  /**
   * Starts profiling a run.
   *
   * @param id - Job identity, as the jobs service minted or adopted it.
   * @param kind - What kind of work it is.
   * @param isPartial - Whether this window joined a run already in progress.
   */
  public begin(id: string, kind: string, isPartial: boolean = false): void {
    this.profiles.set(id, {
      id,
      kind,
      isPartial,
      isFinished: false,
      samples: 0,
      duration: 0,
      phases: [],
      peakRate: 0,
      peakUnit: null,
      longestStall: 0,
      activePhase: null,
      lastDuration: 0,
      lastCompleted: 0,
      stalledFor: 0,
    });

    this.forgetBeyondLimit();
  }

  /**
   * Folds one progress report into the run's profile.
   *
   * @param id - Job the report belongs to.
   * @param progress - The report, as it arrived on the channel.
   */
  public sample(id: string, progress: JobProgress): void {
    const state: Optional<IJobProfileState> = this.profiles.get(id);

    if (!state) {
      return;
    }

    const leaf: Maybe<ProgressLevel> = progress.levels[progress.levels.length - 1];

    state.samples += 1;
    state.duration = progress.duration;

    // The first report closes no slice: there is nothing before it to attribute the time to.
    if (state.activePhase && progress.duration > state.lastDuration) {
      this.accrue(state, progress.duration - state.lastDuration, leaf);
    }

    state.lastDuration = progress.duration;

    if (leaf) {
      state.activePhase = this.phaseOf(state, leaf);
      state.lastCompleted = leaf.completed;
    }
  }

  /**
   * Marks a run as ended.
   *
   * @param id - Job that settled.
   */
  public finish(id: string): void {
    const state: Optional<IJobProfileState> = this.profiles.get(id);

    if (state) {
      state.isFinished = true;
    }
  }

  /**
   * @param id - Job to describe.
   * @returns What this window saw of it, or null for a run it never watched.
   */
  public read(id: string): Nullable<IJobProfile> {
    const state: Optional<IJobProfileState> = this.profiles.get(id);

    return state ? JobProfileRecorder.toProfile(state) : null;
  }

  /** @returns Every profile held, oldest first, copied so a render cannot see one move underneath it. */
  public list(): Array<IJobProfile> {
    return Array.from(this.profiles.values(), JobProfileRecorder.toProfile);
  }

  /** Forgets every profile, which the listing beside it cannot do to the backend's own retention. */
  public reset(): void {
    this.profiles.clear();
  }

  /**
   * Adds one elapsed slice to the phase that was running through it.
   *
   * @param state - Profile being filled in.
   * @param elapsed - Milliseconds since the previous report.
   * @param leaf - The innermost level of this report, where it has one.
   */
  private accrue(state: IJobProfileState, elapsed: number, leaf: Maybe<ProgressLevel>): void {
    const phase: IJobPhase = state.activePhase as IJobPhase;

    phase.duration += elapsed;

    // A counter that restarted belongs to a phase that changed, and a negative delta is not progress.
    const isSamePhase: boolean = Boolean(leaf) && leaf?.id === phase.id;
    const completed: number = isSamePhase ? Math.max(0, (leaf?.completed ?? 0) - state.lastCompleted) : 0;

    phase.completed += completed;

    if (completed > 0) {
      const rate: number = (completed / elapsed) * 1000;

      if (rate > state.peakRate) {
        state.peakRate = rate;
        state.peakUnit = phase.unit;
      }

      state.stalledFor = 0;
    } else {
      state.stalledFor += elapsed;
      state.longestStall = Math.max(state.longestStall, state.stalledFor);
    }
  }

  /**
   * @param state - Profile being filled in.
   * @param leaf - The innermost level of the report just received.
   * @returns Its phase, entered now when this is the first report from it.
   */
  private phaseOf(state: IJobProfileState, leaf: ProgressLevel): IJobPhase {
    const known: Optional<IJobPhase> = state.phases.find((it: IJobPhase) => it.id === leaf.id);

    if (known) {
      return known;
    }

    const phase: IJobPhase = {
      id: leaf.id,
      label: leaf.label,
      duration: 0,
      completed: 0,
      unit: leaf.unit,
    };

    state.phases.push(phase);

    return phase;
  }

  /** Drops the oldest profiles once there are more than the backend keeps listings for. */
  private forgetBeyondLimit(): void {
    while (this.profiles.size > RETAINED_PROFILES) {
      const oldest: Optional<string> = this.profiles.keys().next().value;

      if (oldest === undefined) {
        return;
      }

      this.profiles.delete(oldest);
    }
  }
}

/**
 * One instance for the window, reached by the jobs service as messages land.
 */
export const JOB_PROFILES: JobProfileRecorder = new JobProfileRecorder();
