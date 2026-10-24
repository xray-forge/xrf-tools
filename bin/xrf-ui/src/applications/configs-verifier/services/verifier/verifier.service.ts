import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { Logger } from "@xrf/logging";

import { describeVerifyOutcome } from "@/applications/configs-verifier/lib/describe-verify-outcome";
import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/ipc/commands/configs";
import { CONFIGS_JOB_VERIFY } from "@/core/ipc/types/xrf-app-configs";
import { LtxProjectVerifyResult } from "@/core/ipc/types/xrf-ltx";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/**
 * The configs verification run and what it found.
 */
@Injectable()
export class VerifierService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<LtxProjectVerifyResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [CONFIGS_JOB_VERIFY]);
  }

  /**
   * Verifies every LTX file the directory exposes.
   *
   * @param directory - Configs directory to work over.
   * @param isDltx - Whether to resolve with the Monolith/Anomaly DLTX patch dialect, which applies any
   *   `mod_<base>_*.ltx` beside a config rather than reading it as a config of its own.
   * @param isStrict - Whether to report what only a case-sensitive file system breaks on too, such as an include
   *   naming a file in another case.
   */
  @ExclusiveFlow("operation")
  public *verify(directory: string, isDltx: boolean, isStrict: boolean): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Verifying:", directory);

    yield* this.operation.run({
      kind: CONFIGS_JOB_VERIFY,
      invoke: (id: string, progress) =>
        configsCommands.verifyDirectory(
          { roots: createRoots([directory]), prefix: null, isDltx, isStrict },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<LtxProjectVerifyResult>): IJobNotice => describeVerifyOutcome(directory, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
