import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeFormatOutcome } from "@/applications/translations-formatter/lib/describe-format-outcome";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationFormatResult } from "@/core/bindings/types/xrf-translation";
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
 * The translations formatting run and what it rewrote.
 */
@Injectable()
export class TranslationsFormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<TranslationFormatResult> = null;

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
      this.jobsService.getJobOfKind(EJobKind.TRANSLATIONS_CHECK_FORMAT) ??
      this.jobsService.getJobOfKind(EJobKind.TRANSLATIONS_FORMAT)
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
   * Normalizes the translation sources under a directory, or reports which are not normalized.
   *
   * Line endings are left to the backend's preserve default: asserting one is a build-gate concern and would make this
   * form a place to change every file in a tree by accident.
   *
   * @param directory - Directory of JSON translation sources.
   * @param isCheck - Whether to report the formatting rather than repair it.
   */
  @LatestFlow("isBusy")
  public *format(directory: string, isCheck: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Formatting translations:", directory, isCheck);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<TranslationFormatResult> = this.jobsService.run<TranslationFormatResult>({
      kind: isCheck ? EJobKind.TRANSLATIONS_CHECK_FORMAT : EJobKind.TRANSLATIONS_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? translationsCommands.checkProjectFormat({ directory, lineEndings: null }, id, progress)
          : translationsCommands.formatProject({ directory, lineEndings: null }, id, progress),
      describe: (outcome: IJobOutcome<TranslationFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });

    this.jobId = run.id;

    try {
      const answered: TranslationFormatResult = yield* call(run.promise);

      this.log.info("Translations formatting finished in:", formatDuration(timer.elapsed()));

      this.result = answered;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Translations formatting error after:", formatDuration(timer.elapsed()), transformed);

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

    if (settled?.kind === EJobKind.TRANSLATIONS_FORMAT || settled?.kind === EJobKind.TRANSLATIONS_CHECK_FORMAT) {
      this.log.info("Adopting the outcome of a run this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<TranslationFormatResult>) ?? null;
      this.error = settled.error;
    }
  }
}
