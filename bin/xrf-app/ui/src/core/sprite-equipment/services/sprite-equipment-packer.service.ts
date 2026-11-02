import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { spriteEquipmentCommands } from "@xrf/ipc/commands/sprite-equipment";
import { SPRITE_EQUIPMENT_JOB_PACK } from "@xrf/ipc/types/xrf-app-sprite-equipment";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { describePackSpriteOutcome } from "@/core/sprite-equipment/lib";

/** Packing shared by the standalone runner and the editor's repack action. */
@Injectable()
export class SpriteEquipmentPackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The sheet is written once at the end; a cancelled pack leaves the output unchanged. */
  public readonly operation: JobOperation<EquipmentSpritePackResult> = new JobOperation(this.log, {
    kinds: [SPRITE_EQUIPMENT_JOB_PACK],
  });

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
  ): TFlow<Nullable<EquipmentSpritePackResult>> {
    this.log.info("Packing equipment editor:", sourcePath, outputPath, systemLtxPath);

    const completion: Nullable<TCommandCompletion<EquipmentSpritePackResult>> = yield* this.operation.run({
      kind: SPRITE_EQUIPMENT_JOB_PACK,
      invoke: (id: string, progress) =>
        spriteEquipmentCommands.packSprite({ sourcePath, outputPath, systemLtxPath, isDltx }, id, progress),
      describe: (outcome: IJobOutcome<EquipmentSpritePackResult>): IJobNotice =>
        describePackSpriteOutcome(outputPath, outcome),
    });

    if (completion?.error) {
      throw completion.error;
    }

    return completion?.result ?? null;
  }
}
