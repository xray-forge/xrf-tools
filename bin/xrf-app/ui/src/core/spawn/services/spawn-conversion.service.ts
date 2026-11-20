import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { spawnCommands } from "@xrf/ipc/commands/spawn";
import {
  ESpawnConversion,
  SPAWN_JOB_PACK,
  SPAWN_JOB_UNPACK,
  SpawnConversionResult,
  SpawnConvertRequest,
} from "@xrf/ipc/types/xrf-app-spawn";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { JobOperation } from "@/core/jobs/lib/job-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
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
   * @param request - The directory holding unpacked chunks, the spawn file written, and whether a file already there
   *   is replaced; the backend refuses otherwise.
   * @returns How the run ended, or null where none started.
   */
  @ExclusiveFlow("operation")
  public *pack(request: SpawnConvertRequest): TFlow<Nullable<TCommandCompletion<SpawnConversionResult>>> {
    const { source, destination } = request;

    this.log.info("Packing spawn:", source, "to", destination);

    return yield* this.operation.run({
      kind: SPAWN_JOB_PACK,
      invoke: (id, progress) => spawnCommands.packFile(request, id, progress),
      describe: (outcome) => describeSpawnConversionOutcome(ESpawnConversion.PACK, source, destination, outcome),
    });
  }

  /**
   * Unpacks a spawn file while no spawn conversion is running.
   *
   * @param request - The packed spawn file, the directory written, and whether one already there is replaced; the
   *   backend refuses otherwise.
   * @returns How the run ended, or null where none started.
   */
  @ExclusiveFlow("operation")
  public *unpack(request: SpawnConvertRequest): TFlow<Nullable<TCommandCompletion<SpawnConversionResult>>> {
    const { source, destination } = request;

    this.log.info("Unpacking spawn:", source, "to", destination);

    return yield* this.operation.run({
      kind: SPAWN_JOB_UNPACK,
      invoke: (id, progress) => spawnCommands.unpackFile(request, id, progress),
      describe: (outcome) => describeSpawnConversionOutcome(ESpawnConversion.UNPACK, source, destination, outcome),
    });
  }
}
