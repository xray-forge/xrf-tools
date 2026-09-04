import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeVerifyOutcome } from "@/applications/configs-verifier/lib/describe-verify-outcome";
import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
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
 * The configs verification run and what it found.
 */
@Injectable()
export class VerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<LtxProjectVerifyResult> = null;

  /** The run this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The run currently going, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(EJobKind.CONFIGS_VERIFY);
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
   * Verifies every LTX file the directory exposes.
   *
   * @param directory - Configs directory to work over.
   * @param isDltx - Whether to resolve with the Monolith/Anomaly DLTX patch dialect, which applies any
   *   `mod_<base>_*.ltx` beside a config rather than reading it as a config of its own.
   */
  @LatestFlow("isBusy")
  public *verify(directory: string, isDltx: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Verifying:", directory);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<LtxProjectVerifyResult> = this.jobsService.run<LtxProjectVerifyResult>({
      kind: EJobKind.CONFIGS_VERIFY,
      invoke: (id: string, progress) =>
        configsCommands.verifyDirectory({ roots: createRoots([directory]), prefix: null, isDltx }, id, progress),
      describe: (outcome: IJobOutcome<LtxProjectVerifyResult>): IJobNotice => describeVerifyOutcome(directory, outcome),
    });

    this.jobId = run.id;

    try {
      const answered: LtxProjectVerifyResult = yield* call(run.promise);

      this.log.info("Verifying finished in:", formatDuration(timer.elapsed()));

      this.result = answered;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Verifying error after:", formatDuration(timer.elapsed()), transformed);

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

    if (settled?.kind === EJobKind.CONFIGS_VERIFY) {
      this.log.info("Adopting the outcome of a run this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<LtxProjectVerifyResult>) ?? null;
      this.error = settled.error;
    }
  }
}
