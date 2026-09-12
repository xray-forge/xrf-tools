import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";

import { EJobKind } from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { IJobDescriptor, IJobRun, IJobSettledPayload, IJobState } from "@/core/jobs/lib/jobs-types";
import { JobsService } from "@/core/jobs/services/jobs";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, TFlow } from "@/lib/mobx";
import { Nullable, Optional } from "@/lib/types/general";

/** A reported failure, or the typed answer a follow-up operation can use. */
export type JobCompletion<T> = { result: T; error: null } | { result: null; error: Error };

/**
 * One application's command state, including a run rediscovered after a reload.
 * The owner delegates with `yield*` inside its flow and forwards settled events; it owns the lifetime.
 * Notifications and backend cancellation remain with `JobsService`.
 */
export class JobOperation<T> {
  @Observable()
  private state: AsyncState<T> = AsyncState.idle<T>();

  @Observable()
  private jobId: Nullable<string> = null;

  @Computed()
  public get job(): Nullable<IJobState> {
    if (this.jobId) {
      return this.jobsService.getJob(this.jobId);
    }

    for (const kind of this.kinds) {
      const job: Nullable<IJobState> = this.jobsService.getJobOfKind(kind);

      if (job) {
        return job;
      }
    }

    return null;
  }

  @Computed()
  public get isRunning(): boolean {
    return this.state.isLoading || this.job !== null;
  }

  @Computed()
  public get result(): Nullable<T> {
    return this.state.value;
  }

  @Computed()
  public get error(): Nullable<string> {
    return this.state.error?.message ?? null;
  }

  /** Clears the displayed outcome without cancelling work in progress. */
  @BoundAction()
  public reset(): void {
    this.state = this.state.isLoading ? this.state.asLoading(null) : this.state.asIdle();
  }

  public constructor(
    private readonly jobsService: JobsService,
    private readonly kinds: ReadonlyArray<EJobKind>,
    private readonly log: Logger
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** Requests cooperative cancellation; partial output remains described by the command's result. */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /** Runs inside the owner's flow so superseding or deactivating it cannot publish a late answer. */
  public *run(descriptor: IJobDescriptor<T>): TFlow<JobCompletion<T>> {
    const timer: Timer = new Timer();

    this.state = this.state.asLoading(null);

    try {
      const run: IJobRun<T> = this.jobsService.run(descriptor);

      this.jobId = run.id;

      const result: T = yield* call(run.promise);

      this.state = this.state.asReady(result);
      this.log.info("Job finished:", descriptor.kind, formatDuration(timer.elapsed()));

      return { result, error: null };
    } catch (caught: unknown) {
      const error: Error = transformError(caught);

      this.state = this.state.asFailed(error, null);
      this.log.error("Job failed:", descriptor.kind, formatDuration(timer.elapsed()), error);

      return { result: null, error };
    } finally {
      // Abandoning the owner stops publication, not the backend job or its terminal notification.
      if (this.state.isLoading) {
        this.state = this.state.asIdle();
      }

      this.jobId = null;
    }
  }

  /** Accepts only this operation's job kinds; an awaited run publishes through its own flow. */
  @BoundAction()
  public adopt(settled: Optional<IJobSettledPayload>): void {
    if (!settled || !this.kinds.some((kind) => kind === settled.kind) || this.jobId !== null) {
      return;
    }

    // The owner pairs these kinds with T from the generated binding. Retained backend results use that same shape.
    const result: Nullable<T> = (settled.result as Nullable<T>) ?? null;

    this.state =
      settled.error === null ? this.state.asReady(result) : this.state.asFailed(new Error(settled.error), result);
  }
}
