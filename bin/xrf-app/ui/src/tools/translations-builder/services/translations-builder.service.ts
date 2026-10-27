import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { translationsCommands } from "@xrf/ipc/commands/translations";
import { TRANSLATIONS_JOB_BUILD } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationBuildResult } from "@xrf/ipc/types/xrf-translation";
import { Logger } from "@xrf/logging";

import { createRoots } from "@/core/assets/lib";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeTranslationBuildOutcome } from "@/tools/translations-builder/lib/describe-build-outcome";

/**
 * The translation build and what it produced.
 *
 * A service rather than component state because the run outlives the view that started it: reloading the window must
 * find the work still going rather than an idle form over it.
 */
@Injectable()
export class TranslationsBuilderService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation leaves written string tables complete; another build supplies those not reached. */
  public readonly operation: JobOperation<TranslationBuildResult> = new JobOperation(this.log, {
    kinds: [TRANSLATIONS_JOB_BUILD],
  });

  /**
   * Compiles every translation source under a tree into per-language string tables.
   *
   * @param sources - Source tree to compile.
   * @param language - Language to build, or `all`.
   * @param outputDir - Directory the string tables are written into.
   * @param isSorted - Whether to sort entries within each table.
   */
  @ExclusiveFlow("operation")
  public *build(sources: string, language: ETranslationLanguage, outputDir: string, isSorted: boolean): TFlow {
    this.log.info("Building translations:", sources, language, outputDir);

    yield* this.operation.run({
      kind: TRANSLATIONS_JOB_BUILD,
      invoke: (id: string, progress) =>
        translationsCommands.buildProject(
          { roots: createRoots([sources]), prefix: null, language, outputDir, isSorted },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TranslationBuildResult>): IJobNotice =>
        describeTranslationBuildOutcome(outputDir, outcome),
    });
  }
}
