import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeGamedataVerifyOutcome } from "@/applications/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { gamedataCommands } from "@/core/bindings/commands/gamedata";
import { GamedataVerifySummary } from "@/core/bindings/types/xrf-app";
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
 * The gamedata verification run and what it found.
 *
 * A service rather than component state because the run outlives the view that started it: a full pass over an
 * installation is minutes of work, and reloading the window must find it still going.
 */
@Injectable()
export class GamedataVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<GamedataVerifySummary> = null;

  /** The run this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The verification currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(EJobKind.GAMEDATA_VERIFY);
  }

  /** Forgets the last verdict, which stops being true the moment the root changes. */
  @BoundAction()
  public reset(): void {
    this.result = null;
    this.error = null;
  }

  /** Stops the run. Nothing was written, so nothing is left behind - only the unrun checks stay unanswered. */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Runs every check this build knows over a gamedata root.
   *
   * @param root - Gamedata root to verify.
   * @param isStrict - Whether a check that would warn should fail instead.
   */
  @LatestFlow("isBusy")
  public *verify(root: string, isStrict: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Verifying gamedata:", root, isStrict);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<GamedataVerifySummary> = this.jobsService.run<GamedataVerifySummary>({
      kind: EJobKind.GAMEDATA_VERIFY,
      // Every check this build knows: narrowing the selection is a refinement worth adding once somebody has watched a
      // full run and knows which one they want to repeat.
      invoke: (id: string, progress) => gamedataCommands.verifyProject({ root, checks: null, isStrict }, id, progress),
      describe: (outcome: IJobOutcome<GamedataVerifySummary>): IJobNotice =>
        describeGamedataVerifyOutcome(root, outcome),
    });

    this.jobId = run.id;

    try {
      this.result = yield* call(run.promise);

      this.log.info("Gamedata verified in:", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Verify error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      this.isBusy = false;
      this.jobId = null;
    }
  }

  /**
   * Shows what a run this window watched rather than started has answered.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Nullable<IJobSettledPayload> = event.payload ?? null;

    if (settled?.kind === EJobKind.GAMEDATA_VERIFY) {
      this.log.info("Adopting the outcome of a run this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<GamedataVerifySummary>) ?? null;
      this.error = settled.error;
    }
  }
}
