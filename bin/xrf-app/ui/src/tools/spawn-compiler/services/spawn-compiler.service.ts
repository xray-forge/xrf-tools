import { Injectable } from "@wirestate/core";
import { BoundAction } from "@wirestate/mobx";
import { ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { spawnCommands } from "@xrf/ipc/commands/spawn";
import { SPAWN_JOB_BUILD, SpawnBuildResult } from "@xrf/ipc/types/xrf-app-spawn";
import { SpawnListedLevel, SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";
import { Logger } from "@xrf/logging";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { describeSpawnBuildOutcome } from "@/tools/spawn-compiler/lib/describe-spawn-build-outcome";

/**
 * Lists the levels a spawn can be built from, and runs a root spawn's build over the ones chosen as a job.
 */
@Injectable()
export class SpawnCompilerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The levels the game data lists, each marked by whether the levels directory holds it built. */
  public readonly levels: CommandOperation<Array<SpawnListedLevel>> = new CommandOperation(this.log);

  /** A cancel stops the build between reading the levels, building and writing; once it writes, it finishes. */
  public readonly operation: JobOperation<SpawnBuildResult> = new JobOperation(this.log, {
    kinds: [SPAWN_JOB_BUILD],
  });

  /** Forgets the listing, for paths it was not made for. */
  @BoundAction()
  public resetLevels(): void {
    this.levels.reset();
  }

  /**
   * Lists the levels the game data's `configs/game_levels.ltx` names. Clears the last listing first, so a failure
   * never offers another folder's levels.
   *
   * @param gamedata - The game data holding the level list.
   * @param levelsRoot - The directory holding the built levels' directories.
   */
  @LatestFlow()
  public *listLevels(gamedata: string, levelsRoot: string): TFlow {
    this.levels.reset();

    yield* this.levels.run({
      name: "List spawn levels",
      subject: levelsRoot,
      command: () => spawnCommands.listLevels(gamedata, levelsRoot),
      failure: "Could not list the levels",
      // A warning: the listing runs as paths are named, unasked, and the form shows why beside the levels.
      failureSeverity: ENotificationSeverity.WARNING,
    });
  }

  /**
   * Builds a root spawn from the levels `request` names and writes it.
   *
   * @param request - The levels, where they and the level list are read from, and the spawn to write.
   */
  @ExclusiveFlow("operation")
  public *build(request: SpawnRootRequest): TFlow {
    this.log.info("Building spawn:", request.name, "from", request.levels ?? [], "into", request.destination);

    yield* this.operation.run({
      kind: SPAWN_JOB_BUILD,
      invoke: (id: string, progress) => spawnCommands.buildSpawn(request, id, progress),
      describe: (outcome: IJobOutcome<SpawnBuildResult>): IJobNotice => describeSpawnBuildOutcome(request, outcome),
    });
  }
}
