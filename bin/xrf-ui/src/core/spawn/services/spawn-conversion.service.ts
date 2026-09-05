import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { spawnCommands } from "@/core/bindings/commands/spawn";
import { SpawnConversionResult } from "@/core/bindings/types/xrf-app";
import { EJobKind, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { describeSpawnConversionOutcome } from "@/core/spawn/lib/describe-spawn-conversion-outcome";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";

/** Standalone spawn conversions share one lane and remain visible through the root jobs service. */
@Injectable()
export class SpawnConversionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<SpawnConversionResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.SPAWN_PACK, EJobKind.SPAWN_UNPACK], this.log);
  }

  /**
   * Packs chunks into a spawn file while no spawn conversion is running.
   *
   * @param source - Directory holding unpacked chunks.
   * @param destination - Output spawn file.
   */
  @ExclusiveFlow("operation")
  public *pack(source: string, destination: string): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    yield* this.operation.run({
      kind: EJobKind.SPAWN_PACK,
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
    if (this.operation.isRunning) {
      return;
    }

    yield* this.operation.run({
      kind: EJobKind.SPAWN_UNPACK,
      invoke: (id, progress) => spawnCommands.unpackFile({ source, destination }, id, progress),
      describe: (outcome) => describeSpawnConversionOutcome("unpack", source, destination, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
