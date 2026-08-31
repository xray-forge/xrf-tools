import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeTranslationBuildOutcome } from "@/applications/translations-builder/lib/describe-build-outcome";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationBuildSummary } from "@/core/bindings/types/xrf-app";
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
 * The translation build and what it produced.
 *
 * A service rather than component state because the run outlives the view that started it: reloading the window must
 * find the work still going rather than an idle form over it.
 */
@Injectable()
export class TranslationsBuilderService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<TranslationBuildSummary> = null;

  /** The build this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

  /**
   * @returns The build currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId
      ? this.jobsService.getJob(this.jobId)
      : this.jobsService.getJobOfKind(EJobKind.TRANSLATIONS_BUILD);
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
   * The string tables already written stay valid and the rest are simply absent, which running it again resolves.
   */
  @BoundAction()
  public cancel(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Compiles every translation source under a tree into per-language string tables.
   *
   * @param sources - Source tree to compile.
   * @param language - Language to build, or `all`.
   * @param outputDir - Directory the string tables are written into.
   * @param isSorted - Whether to sort entries within each table.
   */
  @LatestFlow("isBusy")
  public *build(sources: string, language: string, outputDir: string, isSorted: boolean): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Building translations:", sources, language, outputDir);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    const run: IJobRun<TranslationBuildSummary> = this.jobsService.run<TranslationBuildSummary>({
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

    this.jobId = run.id;

    try {
      this.result = yield* call(run.promise);

      this.log.info("Translations built in:", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Build error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      this.isBusy = false;
      this.jobId = null;
    }
  }

  /**
   * Shows what a build this window watched rather than started has answered.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    const settled: Nullable<IJobSettledPayload> = event.payload ?? null;

    if (settled?.kind === EJobKind.TRANSLATIONS_BUILD) {
      this.log.info("Adopting the outcome of a build this window did not start:", settled.id, settled.conclusion);

      this.result = (settled.result as Nullable<TranslationBuildSummary>) ?? null;
      this.error = settled.error;
    }
  }
}
