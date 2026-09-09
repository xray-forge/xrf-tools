import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";

import { spriteEquipmentCommands } from "@/core/bindings/commands/sprite-equipment";
import { EJobKind } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobCompletion, JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { IPackEquipmentResult } from "@/core/sprite-equipment/equipment";
import { describePackSpriteOutcome } from "@/core/sprite-equipment/lib/describe-pack-sprite-outcome";
import { Logger } from "@/lib/logging";
import { ExclusiveFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** Packing shared by the standalone runner and the editor's repack action. */
@Injectable()
export class SpriteEquipmentPackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The sheet is written once at the end; a cancelled pack leaves the output unchanged. */
  public readonly operation: JobOperation<IPackEquipmentResult>;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.SPRITE_EQUIPMENT_PACK], this.log);
  }

  /**
   * Draws every declared icon into one sprite sheet.
   *
   * Started through the jobs service rather than invoked here: reading `system.ltx` pulls in the whole include tree and
   * every icon is decoded, so the run wants an identity, a lease over the sheet it writes, and a way to stop it.
   *
   * @param sourcePath - Directory of individual icon files.
   * @param outputPath - File the sheet is written to.
   * @param systemLtxPath - `system.ltx` declaring which icons exist and where they sit.
   * @param isDltx - Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect.
   * @returns What the run produced, or null when another scope already owns a pack.
   */
  @ExclusiveFlow("operation")
  public *packEquipmentSprite(
    sourcePath: string,
    outputPath: string,
    systemLtxPath: string,
    isDltx: boolean
  ): TFlow<Nullable<IPackEquipmentResult>> {
    if (this.operation.isRunning) {
      return null;
    }

    this.log.info("Packing equipment editor:", sourcePath, outputPath, systemLtxPath);

    const completion: JobCompletion<IPackEquipmentResult> = yield* this.operation.run({
      kind: EJobKind.SPRITE_EQUIPMENT_PACK,
      invoke: (id: string, progress) =>
        spriteEquipmentCommands.packSprite({ sourcePath, outputPath, systemLtxPath, isDltx }, id, progress),
      describe: (outcome: IJobOutcome<IPackEquipmentResult>): IJobNotice =>
        describePackSpriteOutcome(outputPath, outcome),
    });

    if (completion.error !== null) {
      throw completion.error;
    }

    return completion.result;
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
  }
}
