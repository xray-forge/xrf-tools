import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { configsCommands } from "@xrf/ipc/commands/configs";
import { CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT } from "@xrf/ipc/types/xrf-app-configs";
import { LtxProjectFormatResult } from "@xrf/ipc/types/xrf-ltx";
import { Logger } from "@xrf/logging";

import { describeFormatOutcome } from "@/applications/configs-formatter/lib/describe-format-outcome";
import { createRoots } from "@/core/assets/lib";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";

/**
 * The configs formatting run and what it rewrote.
 */
@Injectable()
export class FormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<LtxProjectFormatResult> = new JobOperation(this.log, {
    kinds: [CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT],
  });

  /**
   * Reports or repairs the formatting of every LTX file the directory exposes.
   *
   * One entry point for both, because they are one user operation with a switch on it: the form offers a mode and the
   * run answers in the same shape either way. What differs is the kind it registers as, which is what keeps a check
   * and a rewrite distinguishable everywhere else.
   *
   * @param directory - Configs directory to work over.
   * @param isCheck - Whether to report the formatting rather than repair it.
   */
  @ExclusiveFlow("operation")
  public *format(directory: string, isCheck: boolean): TFlow {
    this.log.info("Formatting:", directory, isCheck);

    yield* this.operation.run({
      kind: isCheck ? CONFIGS_JOB_CHECK_FORMAT : CONFIGS_JOB_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? configsCommands.checkDirectoryFormat({ roots: createRoots([directory]), prefix: null }, id, progress)
          : configsCommands.formatDirectory({ roots: createRoots([directory]), prefix: null }, id, progress),
      describe: (outcome: IJobOutcome<LtxProjectFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });
  }
}
