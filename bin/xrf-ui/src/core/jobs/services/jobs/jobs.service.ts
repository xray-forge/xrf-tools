import { Channel } from "@tauri-apps/api/core";
import { EventBus, inject, Injectable } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { jobsCommands } from "@/core/bindings/commands/jobs";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { transformError } from "@/core/error/lib";
import { IJobDescriptor, IJobRun, IJobState } from "@/core/jobs/lib/jobs-types";
import { emitNotification } from "@/core/notifications/lib";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Every backend job this window started, while it is running.
 */
@Injectable()
export class JobsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Jobs currently running, in the order they were started. */
  @Observable()
  public jobs: Array<IJobState> = [];

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Starts a job and begins watching it.
   *
   * The identity is minted here and never reused, because a cancel aimed at a previous run must not be able to land on
   * its successor. The observable entry is created before the command is sent, so a caller that renders from `jobs`
   * cannot see a gap between asking and appearing.
   *
   * @param descriptor - What to run, what to call it, and what to say when it ends.
   * @returns The job's identity now, and its answer when it settles.
   */
  public run<T>(descriptor: IJobDescriptor<T>): IJobRun<T> {
    const id: string = crypto.randomUUID();
    const progress: Channel<JobProgress> = new Channel<JobProgress>();

    progress.onmessage = (message: JobProgress): void => this.onProgress(id, message);

    this.onStarted(id, descriptor.kind);

    this.log.info("Starting job:", descriptor.kind, id);

    const promise: Promise<T> = descriptor.invoke(id, progress).then(
      (result: T): T => {
        this.onSettled(id, descriptor, result, null);

        return result;
      },
      (error: unknown): never => {
        this.onSettled(id, descriptor, null, transformError(error));

        throw error;
      }
    );

    return { id, promise };
  }

  /**
   * Asks a running job to stop.
   *
   * Cooperative: the run ends at a boundary it chooses, so this marks the request and the job keeps reporting until it
   * actually stops. Showing that gap is the point — a control that appeared to do nothing would be pressed again.
   *
   * @param id - Job to stop.
   */
  @BoundAction()
  public cancel(id: string): void {
    this.log.info("Cancelling job:", id);

    this.jobs = this.jobs.map((job: IJobState) => (job.id === id ? { ...job, isCancelRequested: true } : job));

    void jobsCommands.cancel(id).catch((error: unknown) => {
      // The job may have finished between the control being pressed and the request arriving, which is ordinary. The
      // run's own answer is what reports the outcome either way.
      this.log.info("Cancel did not reach a running job:", id, error);
    });
  }

  /**
   * Finds a running job of one kind.
   *
   * How a tool re-attaches after its view was torn down and rebuilt: the tool's own service died with the container,
   * so the kind is the handle that survives. A tool that did not do this would show an idle form over a run that is
   * still writing files, and let the user start a second one.
   *
   * @param kind - Kind of work to look for.
   * @returns The running job of that kind, or null.
   */
  public getJobOfKind(kind: string): Nullable<IJobState> {
    return this.jobs.find((job: IJobState) => job.kind === kind) ?? null;
  }

  /**
   * @param id - Job to look for.
   * @returns The running job, or null once it has settled.
   */
  public getJob(id: string): Nullable<IJobState> {
    return this.jobs.find((job: IJobState) => job.id === id) ?? null;
  }

  @BoundAction()
  private onStarted(id: string, kind: string): void {
    this.jobs = [...this.jobs, { id, kind, progress: null, isCancelRequested: false }];
  }

  @BoundAction()
  private onProgress(id: string, progress: JobProgress): void {
    // A snapshot for a job no longer listed is one that raced its own settling. Dropping it is right: the entry it
    // would update is gone, and re-adding it would resurrect a finished job.
    this.jobs = this.jobs.map((job: IJobState) => (job.id === id ? { ...job, progress } : job));
  }

  /**
   * Removes the job and says how it went, exactly once.
   *
   * The notification is emitted here rather than by the tool because the tool's container may be long gone by now,
   * and a run finishing in silence is the worst outcome for the one case where the user is not watching it.
   */
  @BoundAction()
  private onSettled<T>(id: string, descriptor: IJobDescriptor<T>, result: Nullable<T>, error: Nullable<Error>): void {
    const job: Nullable<IJobState> = this.getJob(id);

    this.jobs = this.jobs.filter((it: IJobState) => it.id !== id);

    emitNotification(this.eventBus, {
      ...descriptor.describe({ isCancelRequested: Boolean(job?.isCancelRequested), result, error }),
      source: descriptor.source,
    });
  }
}
