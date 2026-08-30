import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeUnpackOutcome } from "@/applications/archives-unpacker/lib/describe-unpack-outcome";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { transformError } from "@/core/error/lib";
import { IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * What an unpack calls itself in the jobs registry.
 */
export const ARCHIVES_UNPACK_JOB_KIND: string = "archives.unpack";

/**
 * The unpacking run and what it produced.
 *
 * A service rather than component state because an unpack outlives the view that started it: a user who navigates
 * away mid-run must be able to come back to it rather than find an idle form over work that is still writing files.
 */
@Injectable()
export class UnpackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<ArchiveUnpackResult> = null;

  /** The unpack this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The unpack currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(ARCHIVES_UNPACK_JOB_KIND);
  }

  /**
   * Forgets the last outcome, which stops being true the moment either path changes.
   */
  @BoundAction()
  public reset(): void {
    this.result = null;
    this.error = null;
  }

  /**
   * Stops the running unpack, if there is one.
   *
   * What it has already written stays on disk: the destination may hold the user's own files, and nothing here can
   * tell those from this run's. The result reports how far it got instead.
   */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Unpacks every archive of a directory into a destination tree.
   *
   * @param source - Directory holding the packed archives.
   * @param destination - Directory the archives are unpacked into.
   */
  @LatestFlow("isBusy")
  public *unpack(source: string, destination: string): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Unpacking:", source);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    // Started through the jobs service rather than invoked here, so the run has an identity the cancel control can
    // reach and survives this view being torn down.
    const run: IJobRun<ArchiveUnpackResult> = this.jobsService.run<ArchiveUnpackResult>({
      kind: ARCHIVES_UNPACK_JOB_KIND,
      source: EApplicationId.ARCHIVES_UNPACKER,
      invoke: (id: string, progress) => archivesCommands.unpackDirectory(source, destination, id, progress),
      describe: (outcome: IJobOutcome<ArchiveUnpackResult>): INotificationPayload =>
        describeUnpackOutcome(source, destination, outcome),
    });

    this.jobId = run.id;

    try {
      const unpacked: ArchiveUnpackResult = yield* call(run.promise);

      this.log.info("Unpacked in:", formatDuration(timer.elapsed()), `(backend ${formatDuration(unpacked.duration)})`);

      this.result = unpacked;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Unpack error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      // Reached on cancellation of this generator too, which is a superseded view rather than a stopped job: the run
      // itself keeps going and reports through the jobs service.
      this.isBusy = false;
      this.jobId = null;
    }
  }
}
