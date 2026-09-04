import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeFormatOutcome } from "@/applications/configs-formatter/lib/describe-format-outcome";
import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { transformError } from "@/core/error/lib";
import {
  EJobKind,
  IJobNotice,
  IJobOutcome,
  IJobRun,
  IJobSettledPayload,
  IJobState,
  JOB_SETTLED_EVENT,
} from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The configs formatting run and what it rewrote.
 */
@Injectable()
export class FormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<LtxProjectFormatResult> = null;

  /** The run this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The run currently going, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    if (this.jobId) {
      return this.jobsService.getJob(this.jobId);
    }

    // Either kind, because this one form runs both and a reload cannot tell which was in flight. Checking is listed
    // first only because it is the mode the form opens in; the two are mutually exclusive in practice, since the lease
    // and the form both admit one run at a time.
    return (
      this.jobsService.getJobOfKind(EJobKind.CONFIGS_CHECK_FORMAT) ??
      this.jobsService.getJobOfKind(EJobKind.CONFIGS_FORMAT)
    );
  }

  /**
   * Forgets the last outcome, which stops being true the moment the directory changes.
   */
  @BoundAction()
  public reset(): void {
    this.result = null;
    this.error = null;
  }

  /**
   * Stops the run, if there is one.
   */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Reports or repairs the formatting of every LTX file the directory exposes.
   *
   * One entry point for both, because they are one user operation with a switch on it: the form offers a mode and the
   * run answers in the same shape either way. What differs is the kind it registers as, which is what keeps a check
   * and a rewrite distinguishable everywhere else.
   *
   * @param directory - Configs directory to work over.
   * @param isCheck - Whether to report the formatting rather than repair it.
   */
  @LatestFlow("isBusy")
  public *format(directory: string, isCheck: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Formatting:", directory, isCheck);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<LtxProjectFormatResult> = this.jobsService.run<LtxProjectFormatResult>({
      kind: isCheck ? EJobKind.CONFIGS_CHECK_FORMAT : EJobKind.CONFIGS_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? configsCommands.checkDirectoryFormat({ roots: createRoots([directory]), prefix: null }, id, progress)
          : configsCommands.formatDirectory({ roots: createRoots([directory]), prefix: null }, id, progress),
      describe: (outcome: IJobOutcome<LtxProjectFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });

    this.jobId = run.id;

    try {
      const answered: LtxProjectFormatResult = yield* call(run.promise);

      this.log.info("Formatting finished in:", formatDuration(timer.elapsed()));

      this.result = answered;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Formatting error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      this.isBusy = false;
      this.jobId = null;
    }
  }

  /**
   * Shows what a run this window watched rather than started has answered.
   *
   * The reload case: the command replied to a page that is gone, so nothing here ever awaited it. What the backend
   * retained arrives instead.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Nullable<IJobSettledPayload> = event.payload ?? null;

    if (settled?.kind === EJobKind.CONFIGS_FORMAT || settled?.kind === EJobKind.CONFIGS_CHECK_FORMAT) {
      this.log.info("Adopting the outcome of a run this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<LtxProjectFormatResult>) ?? null;
      this.error = settled.error;
    }
  }
}
