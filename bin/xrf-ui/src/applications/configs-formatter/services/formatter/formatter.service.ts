import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { describeFormatOutcome } from "@/applications/configs-formatter/lib/describe-format-outcome";
import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { EJobKind, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The configs formatting run and what it rewrote.
 */
@Injectable()
export class FormatterService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<LtxProjectFormatResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.CONFIGS_CHECK_FORMAT, EJobKind.CONFIGS_FORMAT], this.log);
  }

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
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Formatting:", directory, isCheck);

    yield* this.operation.run({
      kind: isCheck ? EJobKind.CONFIGS_CHECK_FORMAT : EJobKind.CONFIGS_FORMAT,
      invoke: (id: string, progress) =>
        isCheck
          ? configsCommands.checkDirectoryFormat({ roots: createRoots([directory]), prefix: null }, id, progress)
          : configsCommands.formatDirectory({ roots: createRoots([directory]), prefix: null }, id, progress),
      describe: (outcome: IJobOutcome<LtxProjectFormatResult>): IJobNotice =>
        describeFormatOutcome(directory, isCheck, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
