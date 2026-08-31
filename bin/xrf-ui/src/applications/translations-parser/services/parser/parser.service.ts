import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeTranslationParseOutcome } from "@/applications/translations-parser/lib/describe-parse-outcome";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationParseSummary } from "@/core/bindings/types/xrf-app";
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
 * The translation import and what it wrote.
 */
@Injectable()
export class TranslationsParserService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<TranslationParseSummary> = null;

  /** The import this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The import currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId
      ? this.jobsService.getJob(this.jobId)
      : this.jobsService.getJobOfKind(EJobKind.TRANSLATIONS_PARSE);
  }

  /** Forgets the last outcome, which stops being true the moment the tree or language changes. */
  @BoundAction()
  public reset(): void {
    this.result = null;
    this.error = null;
  }

  /**
   * Stops the run, if there is one.
   *
   * Each source is written whole through a staged replace, so what it wrote stays complete and the rest are untouched.
   */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
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
  @LatestFlow("isBusy")
  public *parse(sources: string, language: string, outputDir: string, isOverwrite: boolean, isDryRun: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Importing translations:", sources, language, outputDir);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<TranslationParseSummary> = this.jobsService.run<TranslationParseSummary>({
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

    this.jobId = run.id;

    try {
      this.result = yield* call(run.promise);

      this.log.info("Translations imported in:", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Import error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      this.isBusy = false;
      this.jobId = null;
    }
  }

  /**
   * Shows what an import this window watched rather than started has answered.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Nullable<IJobSettledPayload> = event.payload ?? null;

    if (settled?.kind === EJobKind.TRANSLATIONS_PARSE) {
      this.log.info("Adopting the outcome of an import this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<TranslationParseSummary>) ?? null;
      this.error = settled.error;
    }
  }
}
