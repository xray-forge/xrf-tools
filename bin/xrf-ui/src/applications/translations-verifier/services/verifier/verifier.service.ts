import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeTranslationVerifyOutcome } from "@/applications/translations-verifier/lib/describe-verify-outcome";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The translation completeness check and what it found.
 */
@Injectable()
export class TranslationsVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<TranslationVerifySummary>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.TRANSLATIONS_VERIFY], this.log);
  }

  /**
   * Checks every translation source under a tree for missing text.
   *
   * @param sources - Source tree to check.
   * @param language - Language to narrow to, or `all`.
   */
  @ExclusiveFlow("operation")
  public *verify(sources: string, language: string): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Verifying translations:", sources, language);

    yield* this.operation.run({
      kind: EJobKind.TRANSLATIONS_VERIFY,
      invoke: (id: string, progress) =>
        translationsCommands.verifyProject({ roots: createRoots([sources]), prefix: null, language }, id, progress),
      describe: (outcome: IJobOutcome<TranslationVerifySummary>): IJobNotice =>
        describeTranslationVerifyOutcome(sources, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
