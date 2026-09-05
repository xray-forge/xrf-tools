import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeTranslationParseOutcome } from "@/applications/translations-parser/lib/describe-parse-outcome";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationParseSummary } from "@/core/bindings/types/xrf-app";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { LatestFlow, TFlow } from "@/lib/mobx";

/**
 * The translation import and what it wrote.
 */
@Injectable()
export class TranslationsParserService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Each source uses a staged replace, so cancellation leaves written sources complete and the rest untouched. */
  public readonly operation: JobOperation<TranslationParseSummary>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.TRANSLATIONS_PARSE], this.log);
  }

  /**
   * Imports one language's raw string tables into JSON sources.
   *
   * @param sources - Tree holding the raw tables.
   * @param language - Language every entry read is filed under.
   * @param outputDir - Directory the JSON sources are written into.
   * @param isOverwrite - Whether incoming text may replace existing text that differs.
   * @param isDryRun - Whether to compute the answer without writing it.
   */
  @LatestFlow()
  public *parse(sources: string, language: string, outputDir: string, isOverwrite: boolean, isDryRun: boolean): TFlow {
    this.log.info("Importing translations:", sources, language, outputDir);

    yield* this.operation.run({
      kind: EJobKind.TRANSLATIONS_PARSE,
      invoke: (id: string, progress) =>
        translationsCommands.parseProject(
          {
            // Mounted through the containing installation rather than the shared root builder: raw string tables are
            // read where the engine would find them, which for an import is commonly a tree inside a game directory.
            roots: { asset: null, roots: [{ path: sources, mode: "containingInstallation" }] },
            language,
            prefix: null,
            outputDir,
            file: null,
            isOverwrite,
            isDryRun,
          },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TranslationParseSummary>): IJobNotice =>
        describeTranslationParseOutcome(outputDir, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
