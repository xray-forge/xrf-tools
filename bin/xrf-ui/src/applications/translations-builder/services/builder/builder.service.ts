import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeTranslationBuildOutcome } from "@/applications/translations-builder/lib/describe-build-outcome";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationBuildSummary } from "@/core/bindings/types/xrf-app";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { LatestFlow, TFlow } from "@/lib/mobx";

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
  public readonly operation: JobOperation<TranslationBuildSummary>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.TRANSLATIONS_BUILD], this.log);
  }

  /**
   * Compiles every translation source under a tree into per-language string tables.
   *
   * @param sources - Source tree to compile.
   * @param language - Language to build, or `all`.
   * @param outputDir - Directory the string tables are written into.
   * @param isSorted - Whether to sort entries within each table.
   */
  @LatestFlow()
  public *build(sources: string, language: string, outputDir: string, isSorted: boolean): TFlow {
    this.log.info("Building translations:", sources, language, outputDir);

    yield* this.operation.run({
      kind: EJobKind.TRANSLATIONS_BUILD,
      invoke: (id: string, progress) =>
        translationsCommands.buildProject(
          { roots: createRoots([sources]), prefix: null, language, outputDir, isSorted },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TranslationBuildSummary>): IJobNotice =>
        describeTranslationBuildOutcome(outputDir, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
