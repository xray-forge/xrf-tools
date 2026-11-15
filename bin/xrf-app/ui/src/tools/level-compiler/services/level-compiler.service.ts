import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelBuildResult, LEVELS_JOB_BUILD } from "@xrf/ipc/types/xrf-app-levels";
import { LevelCompileRequest } from "@xrf/ipc/types/xrf-level-compiler";
import { LevelBuildRequest } from "@xrf/ipc/types/xrf-rawdata";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeBuildOutcome } from "@/tools/level-compiler/lib/describe-build-outcome";

/**
 * Runs a scene's build into a level directory, and the compile after it, as one job.
 */
@Injectable()
export class LevelCompilerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** A cancel stops the build before it writes, or the compile within its running step, keeping what it wrote. */
  public readonly operation: JobOperation<LevelBuildResult> = new JobOperation(this.log, {
    kinds: [LEVELS_JOB_BUILD],
  });

  /**
   * Builds a Level Editor scene into a level directory, then compiles it where steps are asked for.
   *
   * @param request - The scene, the directory to write and how.
   * @param compile - The steps to compile the written level with, and what the compilers read; none to build alone.
   */
  @ExclusiveFlow("operation")
  public *build(request: LevelBuildRequest, compile: Nullable<LevelCompileRequest>): TFlow {
    this.log.info("Building:", request.source, "to", request.destination, "compiling:", compile?.steps ?? []);

    yield* this.operation.run({
      kind: LEVELS_JOB_BUILD,
      invoke: (id: string, progress) => levelsCommands.buildLevel(request, compile, id, progress),
      describe: (outcome: IJobOutcome<LevelBuildResult>): IJobNotice => describeBuildOutcome(request, outcome),
    });
  }
}
