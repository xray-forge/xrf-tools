import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelBuildResult, LEVELS_JOB_BUILD } from "@xrf/ipc/types/xrf-app-levels";
import { LevelBuildRequest } from "@xrf/ipc/types/xrf-rawdata";
import { Logger } from "@xrf/logging";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeBuildOutcome } from "@/tools/level-compiler/lib/describe-build-outcome";

/**
 * The build run and what it wrote.
 */
@Injectable()
export class LevelCompilerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Cancellation stops between stages, before anything is written. */
  public readonly operation: JobOperation<LevelBuildResult> = new JobOperation(this.log, {
    kinds: [LEVELS_JOB_BUILD],
  });

  /**
   * Builds a Level Editor scene into a level directory.
   *
   * @param request - The scene, the directory to write and how.
   */
  @ExclusiveFlow("operation")
  public *build(request: LevelBuildRequest): TFlow {
    this.log.info("Building:", request.source, "to", request.destination);

    yield* this.operation.run({
      kind: LEVELS_JOB_BUILD,
      invoke: (id: string, progress) => levelsCommands.buildLevel(request, id, progress),
      describe: (outcome: IJobOutcome<LevelBuildResult>): IJobNotice => describeBuildOutcome(request, outcome),
    });
  }
}
