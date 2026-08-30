import { Channel } from "@tauri-apps/api/core";
import { EventBus, inject, Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { jobsCommands } from "@/core/bindings/commands/jobs";
import { JobConclusion, JobDescription } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { transformError } from "@/core/error/lib";
import { describeAdoptedOutcome } from "@/core/jobs/lib/describe-adopted-outcome";
import { IJobDescriptor, IJobRun, IJobSettledPayload, IJobState, JOB_SETTLED_EVENT } from "@/core/jobs/lib/jobs-types";
import { emitNotification } from "@/core/notifications/lib";
import { Logger } from "@/lib/logging";
import { all, call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * Every backend job this window started, while it is running.
 */
@Injectable()
export class JobsService {
  /** How often an adopted job is asked where it has got to. */
  private static readonly ADOPTION_POLL_INTERVAL: number = 1000;

  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Jobs currently running, in the order they were started. */
  @Observable()
  public jobs: Array<IJobState> = [];

  /** Timer watching adopted jobs, running only while there is one to watch. */
  private adoptionTimer: Nullable<ReturnType<typeof setInterval>> = null;

  /** Adopted jobs now reporting to a channel of this window's own, rather than to the page that started them. */
  private attached: Set<string> = new Set();

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Finds jobs the backend is already running and watches them.
   * The window can be reloaded while work is in flight.
   */
  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    await flowResult(this.adoptRunningJobs());
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);

    // An adoption still in flight belongs to the provision that is ending. Letting it land would take jobs over on
    // behalf of a window that has gone, and start a timer nothing would ever stop.
    cancelFlow(this, "jobs");
    this.stopWatchingAdopted();
  }

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
    // No request: the tool that started this has its own arguments, and echoing them back through the backend would
    // be a second copy of the truth for the one case that does not need it.
    this.jobs = [...this.jobs, { id, kind, progress: null, request: null, isCancelRequested: false, isAdopted: false }];
  }

  /**
   * Takes over whatever the backend is already running.
   *
   * Failure is not reported: a listing this window could not read is a worse reason to interrupt a start-up than the
   * jobs it would have described, and the lease still prevents a duplicate from doing damage.
   */
  @LatestFlow("jobs")
  private *adoptRunningJobs(): TFlow {
    const listed: Array<JobDescription> = yield* call(this.listJobs());
    const running: Array<JobDescription> = listed.filter((job: JobDescription) => job.conclusion === null);

    if (!running.length) {
      return;
    }

    this.log.info("Adopting jobs already running:", running.length);

    this.onAdopted(running);

    // Awaited rather than left running, so provisioning finishes with this window actually watching. It also keeps the
    // attaches inside the lane: a provision superseded here stops before it can point a job at a page that is going.
    yield* all(running.map((job: JobDescription) => this.attachTo(job.id)));

    this.startWatchingAdopted();
  }

  /**
   * Asks a running job to report to this window from now on.
   *
   * The job is still writing snapshots to the channel of the page that started it, which no longer exists - every one
   * of them is a callback the webview cannot find, ten times a second, until the run ends. A channel of this window's
   * own replaces it, and the listing goes back to being what it is for: finding jobs and seeing them finish.
   *
   * Failure leaves the job adopted and polled. That is a slower bar rather than a broken one, and it is not worth
   * interrupting a start-up over.
   *
   * @param id - Job to watch.
   */
  private async attachTo(id: string): Promise<void> {
    if (this.attached.has(id)) {
      return;
    }

    const progress: Channel<JobProgress> = new Channel<JobProgress>();

    progress.onmessage = (message: JobProgress): void => this.onProgress(id, message);

    try {
      if (await jobsCommands.attach(id, progress)) {
        this.attached.add(id);
      }
    } catch (error: unknown) {
      this.log.error("Could not watch an adopted job:", id, error);
    }
  }

  /**
   * Asks the backend where the adopted jobs have got to, and settles the ones that have ended.
   *
   * Only adopted jobs are updated from a listing. A job this window started is already receiving its own snapshots
   * through a channel, and a listing can lag those by up to one emission interval - merging the two would let a
   * running bar go backwards.
   */
  @LatestFlow()
  private *pollAdoptedJobs(): TFlow {
    const listed: Array<JobDescription> = yield* call(this.listJobs());
    const byId: Map<string, JobDescription> = new Map(listed.map((job: JobDescription) => [job.id, job]));

    for (const job of this.jobs.filter((it: IJobState) => it.isAdopted)) {
      const described: JobDescription | undefined = byId.get(job.id);

      // Absent from the listing at all means it finished and then fell out of the retained ring, which takes twenty
      // finished jobs. It ended; how is no longer recorded.
      if (!described) {
        this.onAdoptedSettled(job, null, null, null);

        continue;
      }

      if (described.conclusion) {
        this.onAdoptedSettled(job, described.conclusion, described.error, described.result);

        continue;
      }

      // Only where nothing better is arriving. An attached job is already receiving its own snapshots, and a listing
      // can lag those by up to one emission interval - applying both would let a running bar go backwards.
      if (!this.attached.has(job.id)) {
        this.onProgress(job.id, described.progress);
      }
    }

    if (!this.jobs.some((job: IJobState) => job.isAdopted)) {
      this.stopWatchingAdopted();
    }
  }

  /**
   * Reads what the backend is running, answering nothing where it cannot be read.
   *
   * A start-up is a bad moment to fail loudly, and a poll that threw would take its own timer down with it. The lease
   * still prevents a job this window could not see from being started twice.
   *
   * @returns Every job the backend describes, or an empty listing.
   */
  private async listJobs(): Promise<Array<JobDescription>> {
    try {
      const listed: Array<JobDescription> = await jobsCommands.list();

      return Array.isArray(listed) ? listed : [];
    } catch (error: unknown) {
      this.log.error("Could not read running jobs:", error);

      return [];
    }
  }

  private startWatchingAdopted(): void {
    // Only where nothing is watching yet. A second timer would poll the same listing twice a second and settle every
    // adopted job twice, announcing each outcome to the user as many times as this was called.
    if (!this.adoptionTimer) {
      this.adoptionTimer = setInterval(() => void this.pollAdoptedJobs(), JobsService.ADOPTION_POLL_INTERVAL);
    }
  }

  private stopWatchingAdopted(): void {
    if (this.adoptionTimer) {
      clearInterval(this.adoptionTimer);
      this.adoptionTimer = null;
    }
  }

  @BoundAction()
  private onAdopted(running: Array<JobDescription>): void {
    // Ignoring what is already here, because provisioning can happen more than once over one service instance - React
    // strict mode remounts the provider, and the container it retains hands back the same service. A job listed again
    // is the same run, and a second entry for it would draw two bars and offer two cancels for one pack.
    const known: Set<string> = new Set(this.jobs.map((job: IJobState) => job.id));

    this.jobs = [
      ...this.jobs,
      ...running
        .filter((job: JobDescription) => !known.has(job.id))
        .map((job: JobDescription) => ({
          id: job.id,
          kind: job.kind,
          progress: job.progress,
          request: job.request,
          isCancelRequested: job.isCancelRequested,
          isAdopted: true,
        })),
    ];
  }

  /**
   * Removes a watched job and says how it went, to the notification centre and to whichever tool owns that kind.
   *
   * Both, because they answer different questions. The notification is for the user, who may be nowhere near the tool
   * by now; the event is for the tool, which can render the answer it was never handed - the command replied to a page
   * that no longer exists, and this is the copy the backend kept.
   */
  @BoundAction()
  private onAdoptedSettled(
    job: IJobState,
    conclusion: Nullable<JobConclusion>,
    error: Nullable<string>,
    result: unknown
  ): void {
    this.jobs = this.jobs.filter((it: IJobState) => it.id !== job.id);
    this.attached.delete(job.id);

    emitNotification(this.eventBus, describeAdoptedOutcome(job, conclusion, error));

    this.eventBus.emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: job.id,
      kind: job.kind,
      conclusion,
      error,
      result,
    });
  }

  @BoundAction()
  private onProgress(id: string, progress: Nullable<JobProgress>): void {
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
