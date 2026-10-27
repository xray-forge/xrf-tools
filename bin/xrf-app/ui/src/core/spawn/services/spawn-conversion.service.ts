import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { spawnCommands } from "@xrf/ipc/commands/spawn";
import { SPAWN_JOB_PACK, SPAWN_JOB_UNPACK, SpawnConversionResult } from "@xrf/ipc/types/xrf-app-spawn";
import { Logger } from "@xrf/logging";

import { JobOperation } from "@/core/jobs/lib/job-operation";
import { describeSpawnConversionOutcome } from "@/core/spawn/lib/describe-spawn-conversion-outcome";

/** Standalone spawn conversions share one lane and remain visible through the root jobs service. */
@Injectable()
export class SpawnConversionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<SpawnConversionResult> = new JobOperation(this.log, {
    kinds: [SPAWN_JOB_PACK, SPAWN_JOB_UNPACK],
  });

  /**
   * Packs chunks into a spawn file while no spawn conversion is running.
   *
   * @param source - Directory holding unpacked chunks.
   * @param destination - Output spawn file.
   */
  @ExclusiveFlow("operation")
  public *pack(source: string, destination: string): TFlow {
    this.log.info("Packing spawn:", source, "to", destination);

    yield* this.operation.run({
      kind: SPAWN_JOB_PACK,
      invoke: (id, progress) => spawnCommands.packFile({ source, destination }, id, progress),
      describe: (outcome) => describeSpawnConversionOutcome("pack", source, destination, outcome),
    });
  }

  /**
   * Unpacks a spawn file while no spawn conversion is running.
   *
   * @param source - Packed spawn file.
   * @param destination - Directory for unpacked chunks.
   */
  @ExclusiveFlow("operation")
  public *unpack(source: string, destination: string): TFlow {
    this.log.info("Unpacking spawn:", source, "to", destination);

    yield* this.operation.run({
      kind: SPAWN_JOB_UNPACK,
      invoke: (id, progress) => spawnCommands.unpackFile({ source, destination }, id, progress),
      describe: (outcome) => describeSpawnConversionOutcome("unpack", source, destination, outcome),
    });
  }
}
