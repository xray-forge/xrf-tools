import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { translationsCommands } from "@xrf/ipc/commands/translations";
import { TRANSLATIONS_JOB_VERIFY } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationVerifyResult } from "@xrf/ipc/types/xrf-translation";
import { Logger } from "@xrf/logging";

import { createRoots } from "@/core/assets/lib";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeTranslationVerifyOutcome } from "@/tools/translations-verifier/lib/describe-verify-outcome";

/**
 * The translation completeness check and what it found.
 */
@Injectable()
export class TranslationsVerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<TranslationVerifyResult> = new JobOperation(this.log, {
    kinds: [TRANSLATIONS_JOB_VERIFY],
  });

  /**
   * Checks every translation source under a tree for missing text.
   *
   * @param sources - Source tree to check.
   * @param language - Language to narrow to, or `all`.
   */
  @ExclusiveFlow("operation")
  public *verify(sources: string, language: ETranslationLanguage): TFlow {
    this.log.info("Verifying translations:", sources, language);

    yield* this.operation.run({
      kind: TRANSLATIONS_JOB_VERIFY,
      invoke: (id: string, progress) =>
        translationsCommands.verifyProject({ roots: createRoots([sources]), prefix: null, language }, id, progress),
      describe: (outcome: IJobOutcome<TranslationVerifyResult>): IJobNotice =>
        describeTranslationVerifyOutcome(sources, outcome),
    });
  }
}
