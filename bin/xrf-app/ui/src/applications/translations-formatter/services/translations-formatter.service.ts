import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { translationsCommands } from "@xrf/ipc/commands/translations";
import { TRANSLATIONS_JOB_CHECK_FORMAT, TRANSLATIONS_JOB_FORMAT } from "@xrf/ipc/types/xrf-app-translations";
import { TranslationFormatResult } from "@xrf/ipc/types/xrf-translation";
import { Logger } from "@xrf/logging";

import { describeFormatOutcome } from "@/applications/translations-formatter/lib/describe-format-outcome";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";

/**
 * The translations formatting run and what it rewrote.
 */
@Injectable()
export class TranslationsFormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<TranslationFormatResult> = new JobOperation(this.log, {
    kinds: [TRANSLATIONS_JOB_CHECK_FORMAT, TRANSLATIONS_JOB_FORMAT],
  });

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
    this.log.info("Formatting translations:", directory, isCheck);

    yield* this.operation.run({
      kind: isCheck ? TRANSLATIONS_JOB_CHECK_FORMAT : TRANSLATIONS_JOB_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? translationsCommands.checkProjectFormat({ directory, lineEndings: null }, id, progress)
          : translationsCommands.formatProject({ directory, lineEndings: null }, id, progress),
      describe: (outcome: IJobOutcome<TranslationFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });
  }
}
