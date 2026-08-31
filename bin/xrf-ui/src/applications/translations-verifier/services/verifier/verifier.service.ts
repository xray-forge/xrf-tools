import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeTranslationVerifyOutcome } from "@/applications/translations-verifier/lib/describe-verify-outcome";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
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
 * The translation completeness check and what it found.
 */
@Injectable()
export class TranslationsVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<TranslationVerifySummary> = null;

  /** The check this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The check currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId
      ? this.jobsService.getJob(this.jobId)
      : this.jobsService.getJobOfKind(EJobKind.TRANSLATIONS_VERIFY);
  }

  /** Forgets the last outcome, which stops being true the moment the tree or language changes. */
  @BoundAction()
  public reset(): void {
    this.result = null;
    this.error = null;
  }

  /** Stops the run, if there is one. Nothing was written, so nothing is left behind. */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Checks every translation source under a tree for missing text.
   *
   * @param sources - Source tree to check.
   * @param language - Language to narrow to, or `all`.
   */
  @LatestFlow("isBusy")
  public *verify(sources: string, language: string): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Verifying translations:", sources, language);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<TranslationVerifySummary> = this.jobsService.run<TranslationVerifySummary>({
      kind: EJobKind.TRANSLATIONS_VERIFY,
      invoke: (id: string, progress) =>
        translationsCommands.verifyProject(createRoots([sources]), null, language, id, progress),
      describe: (outcome: IJobOutcome<TranslationVerifySummary>): IJobNotice =>
        describeTranslationVerifyOutcome(sources, outcome),
    });

    this.jobId = run.id;

    try {
      this.result = yield* call(run.promise);

      this.log.info("Translations verified in:", formatDuration(timer.elapsed()));
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
   * Shows what a check this window watched rather than started has answered.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Nullable<IJobSettledPayload> = event.payload ?? null;

    if (settled?.kind === EJobKind.TRANSLATIONS_VERIFY) {
      this.log.info("Adopting the outcome of a check this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<TranslationVerifySummary>) ?? null;
      this.error = settled.error;
    }
  }
}
