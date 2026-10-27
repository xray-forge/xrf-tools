import { EventBus, EventUnsubscribe, inject, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, TFlow } from "@xrf/flow";
import { IScopedSession, SessionScopeService } from "@xrf/ipc/session";
import { JobKind } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { Nullable, Optional } from "@xrf/types";

import { JobsService } from "@/core/jobs/services/jobs.service";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";

import { IJobDescriptor, IJobRun, IJobSettledPayload, IJobState, JOB_SETTLED_EVENT } from "./jobs-types";

/** What a job operation runs and follows. */
export interface IJobOperationOptions<T> {
  /** The kinds of work it starts, and whose settled outcomes it shows when it did not await them. */
  kinds: ReadonlyArray<JobKind>;
  /**
   * Told after an outcome this operation did not await is shown, the reload and navigation cases.
   *
   * @param result - What the backend retained of the answer, or null where it kept none.
   */
  adopted?: (result: Nullable<T>) => void;
}

/**
 * One tool's command state, including a run rediscovered after a reload.
 *
 * The owner delegates with `yield*` inside its flow; the operation ignores a run while one of its kinds is in flight,
 * and shows an outcome it did not await as the jobs service announces it, following those announcements until its
 * tool's scope is released. Notifications and backend cancellation remain with `JobsService`.
 */
export class JobOperation<T> implements IScopedSession {
  @Observable()
  private state: AsyncState<T> = AsyncState.idle<T>();

  @Observable()
  private jobId: Nullable<string> = null;

  private unsubscribe: Nullable<EventUnsubscribe> = null;

  /**
   * @param log - The owning service's logger.
   * @param options - The kinds it runs and what to tell the owner.
   * @param jobsService - Runs the jobs and lists the running ones.
   * @param scopeService - The tool's scope, whose release stops the following.
   * @param eventBus - The bus settled jobs are announced on.
   */
  public constructor(
    private readonly log: Logger,
    private readonly options: IJobOperationOptions<T>,
    private readonly jobsService: JobsService = inject(JobsService),
    scopeService: SessionScopeService = inject(SessionScopeService),
    eventBus: EventBus = inject(EventBus)
  ) {
    scopeService.register(this);

    // Composed state is not activated by the container's observable plugin; it binds the handler subscribed below.
    makeObservable(this);

    this.unsubscribe = eventBus.subscribe(JOB_SETTLED_EVENT, this.onJobSettled);
  }

  @Computed()
  public get job(): Nullable<IJobState> {
    if (this.jobId) {
      return this.jobsService.getJob(this.jobId);
    }

    for (const kind of this.options.kinds) {
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

  /** Clears a reported failure while retaining its result and leaving active work alone. */
  @BoundAction()
  public clearError(): void {
    if (this.state.isFailed) {
      this.state = this.state.asIdle(this.state.value);
    }
  }

  /** Requests cooperative cancellation; partial output remains described by the command's result. */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Runs inside the owner's flow so superseding or deactivating it cannot publish a late answer.
   *
   * @param descriptor - What to run.
   * @returns How the run ended, or null where one of its kinds was already in flight, this window's or one found
   *   running after a reload, and nothing was started.
   */
  public *run(descriptor: IJobDescriptor<T>): TFlow<Nullable<TCommandCompletion<T>>> {
    if (this.isRunning) {
      this.log.info("Ignoring a run while one is in flight:", descriptor.kind);

      return null;
    }

    this.state = this.state.asLoading(null);

    try {
      const run: IJobRun<T> = this.jobsService.run(descriptor);

      this.jobId = run.id;

      const result: T = yield* call(run.promise);

      this.state = this.state.asReady(result);

      return { result, error: null };
    } catch (caught: unknown) {
      const error: Error = transformError(caught);

      if (this.jobId === null) {
        this.log.error("Failed to start job:", descriptor.kind, error);
      }

      this.state = this.state.asFailed(error, null);

      return { result: null, error };
    } finally {
      // Abandoning the owner stops publication, not the backend job or its terminal notification.
      if (this.state.isLoading) {
        this.state = this.state.asIdle();
      }

      this.jobId = null;
    }
  }

  /** Stops following settled jobs as the tool is left. */
  public retire(): void {
    this.unsubscribe?.();
    this.unsubscribe = null;
  }

  /**
   * Shows an outcome of this operation's kinds that it did not await; an awaited run publishes through its own flow.
   *
   * @param event - The settled job the jobs service announced.
   */
  @BoundAction()
  private onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Optional<IJobSettledPayload> = event.payload;

    if (!settled || !this.options.kinds.some((kind) => kind === settled.kind) || this.jobId !== null) {
      return;
    }

    // The owner pairs these kinds with T from the generated binding. Retained backend results use that same shape.
    const result: Nullable<T> = (settled.result as Nullable<T>) ?? null;

    this.state =
      settled.error === null ? this.state.asReady(result) : this.state.asFailed(new Error(settled.error), result);

    this.options.adopted?.(result);
  }
}
