import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { translationsCommands } from "@xrf/ipc/commands/translations";
import { TRANSLATIONS_JOB_PARSE } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationParseResult } from "@xrf/ipc/types/xrf-translation";
import { Logger } from "@xrf/logging";

import { describeTranslationParseOutcome } from "@/applications/translations-parser/lib/describe-parse-outcome";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The translation import and what it wrote.
 */
@Injectable()
export class TranslationsParserService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Each source uses a staged replace, so cancellation leaves written sources complete and the rest untouched. */
  public readonly operation: JobOperation<TranslationParseResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [TRANSLATIONS_JOB_PARSE]);
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
  @ExclusiveFlow("operation")
  public *parse(
    sources: string,
    language: ETranslationLanguage,
    outputDir: string,
    isOverwrite: boolean,
    isDryRun: boolean
  ): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Importing translations:", sources, language, outputDir);

    yield* this.operation.run({
      kind: TRANSLATIONS_JOB_PARSE,
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
      describe: (outcome: IJobOutcome<TranslationParseResult>): IJobNotice =>
        describeTranslationParseOutcome(outputDir, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
