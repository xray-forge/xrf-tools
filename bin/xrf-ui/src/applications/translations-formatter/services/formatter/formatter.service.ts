import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeFormatOutcome } from "@/applications/translations-formatter/lib/describe-format-outcome";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { EJobKind } from "@/core/bindings/types/xrf-app";
import { TranslationFormatResult } from "@/core/bindings/types/xrf-translation";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The translations formatting run and what it rewrote.
 */
@Injectable()
export class TranslationsFormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<TranslationFormatResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(
      jobsService,
      [EJobKind.TRANSLATIONS_CHECK_FORMAT, EJobKind.TRANSLATIONS_FORMAT],
      this.log
    );
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
  @ExclusiveFlow("operation")
  public *format(directory: string, isCheck: boolean): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Formatting translations:", directory, isCheck);

    yield* this.operation.run({
      kind: isCheck ? EJobKind.TRANSLATIONS_CHECK_FORMAT : EJobKind.TRANSLATIONS_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? translationsCommands.checkProjectFormat({ directory, lineEndings: null }, id, progress)
          : translationsCommands.formatProject({ directory, lineEndings: null }, id, progress),
      describe: (outcome: IJobOutcome<TranslationFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
