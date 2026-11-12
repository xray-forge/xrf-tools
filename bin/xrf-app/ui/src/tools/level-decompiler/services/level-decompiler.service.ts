import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelDecompileResult, LEVELS_JOB_DECOMPILE } from "@xrf/ipc/types/xrf-app-levels";
import { LevelDecompileRequest } from "@xrf/ipc/types/xrf-rawdata";
import { Logger } from "@xrf/logging";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeDecompileOutcome } from "@/tools/level-decompiler/lib/describe-decompile-outcome";

/**
 * The decompile run and what it wrote.
 */
@Injectable()
export class LevelDecompilerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation stops between steps, before anything is written. */
  public readonly operation: JobOperation<LevelDecompileResult> = new JobOperation(this.log, {
    kinds: [LEVELS_JOB_DECOMPILE],
  });

  /**
   * Decompiles a compiled level into a Level Editor scene on disk.
   *
   * @param request - The level, the scene to write and how.
   */
  @ExclusiveFlow("operation")
  public *decompile(request: LevelDecompileRequest): TFlow {
    this.log.info("Decompiling:", request.source, "to", request.destination);

    yield* this.operation.run({
      kind: LEVELS_JOB_DECOMPILE,
      invoke: (id: string, progress) => levelsCommands.decompileLevel(request, id, progress),
      describe: (outcome: IJobOutcome<LevelDecompileResult>): IJobNotice => describeDecompileOutcome(request, outcome),
    });
  }
}
