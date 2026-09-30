import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { fetchBulk } from "@/core/ipc/bulk";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsBulkRoutes } from "@/core/ipc/commands/levels-bulk";
import {
  LevelSpawnModelDescription,
  LevelSpawnModelFailure,
  LevelSpawnModelsDescription,
  LevelSpawnObjectsDescription,
  LevelTextureReference,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { ILevelSpawnDelivery, ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";

/** Visuals described a call: a level's few hundred take a dozen calls, and the first objects stand early. */
export const LEVEL_SPAWN_BATCH: number = 24;

/** What a spawn read needs of whoever owns the level, so the reader owns none of it. */
export interface ILevelSpawnReaderHost {
  /** Whether that level opening is still the one held, checked after every step. */
  isOpen(sessionId: string): boolean;
  /**
   * Supplies the textures the models read so far bind, claimed ahead of them so a settle meanwhile keeps them.
   *
   * @param sessionId - The level opening the models belong to.
   * @param textures - Everything the models read so far bind.
   */
  supply(sessionId: string, textures: ReadonlyArray<LevelTextureReference>): Promise<void>;
  /** Takes the objects and every model read so far, for a level still open. */
  deliver(delivery: ILevelSpawnDelivery): void;
  /** Takes how far the read has got. */
  note(report: ILevelSpawnReport): void;
}

/** What one batch of visuals came to. */
interface ILevelSpawnBatch {
  models: Array<ILevelSpawnModel>;
  failures: Array<LevelSpawnModelFailure>;
}

/**
 * Reads a level's spawned objects, then the models they stand as a batch at a time, handing each batch on as it
 * arrives.
 */
export class LevelSpawnReader {
  private readonly host: ILevelSpawnReaderHost;

  public constructor(host: ILevelSpawnReaderHost) {
    this.host = host;
  }

  /**
   * @param sessionId - The level opening to read the spawn of.
   * @returns What the read came to, or null for a level drawing no object or closed meanwhile.
   */
  public async read(sessionId: string): Promise<Nullable<ILevelSpawnReport>> {
    const { value: objects }: SessionSnapshot<LevelSpawnObjectsDescription> =
      await levelsCommands.openSpawnObjects(sessionId);

    if (!objects.visuals.length || !this.host.isOpen(sessionId)) {
      return null;
    }

    const indices: Map<string, number> = new Map(objects.visuals.map((name: string, index: number) => [name, index]));
    const models: Map<number, ILevelSpawnModel> = new Map();
    const textures: Array<LevelTextureReference> = [];
    const failures: Array<LevelSpawnModelFailure> = [];
    let report: ILevelSpawnReport = {
      failures,
      objects: objects.objects.length,
      read: 0,
      visuals: objects.visuals.length,
    };

    this.host.note(report);

    for (let start: number = 0; start < objects.visuals.length; start += LEVEL_SPAWN_BATCH) {
      const names: Array<string> = objects.visuals.slice(start, start + LEVEL_SPAWN_BATCH);
      const batch: ILevelSpawnBatch = await this.readBatch(sessionId, names);

      if (!this.host.isOpen(sessionId)) {
        return null;
      }

      textures.push(...batch.models.flatMap((model: ILevelSpawnModel) => model.description.textures));
      await this.host.supply(sessionId, textures);

      if (!this.host.isOpen(sessionId)) {
        return null;
      }

      for (const model of batch.models) {
        const visual: Maybe<number> = indices.get(model.description.name);

        if (visual !== undefined) {
          models.set(visual, model);
        }
      }

      failures.push(...batch.failures);
      report = { ...report, failures: [...failures], read: start + names.length };
      this.host.deliver({ models: new Map(models), objects });
      this.host.note(report);
    }

    return report;
  }

  /** One batch's models and their packs; a batch that could not be read is every one of its visuals unreadable. */
  private async readBatch(sessionId: string, names: ReadonlyArray<string>): Promise<ILevelSpawnBatch> {
    try {
      const { value }: SessionSnapshot<LevelSpawnModelsDescription> = await levelsCommands.describeSpawnModels(
        sessionId,
        names.slice()
      );
      const buffers: Array<ArrayBuffer> = await Promise.all(
        value.models.map((model: LevelSpawnModelDescription) =>
          fetchBulk(levelsBulkRoutes.readSpawnModel(sessionId, model.name))
        )
      );
      const batch: ILevelSpawnBatch = { failures: value.failures, models: [] };

      value.models.forEach((description: LevelSpawnModelDescription, index: number) => {
        const buffer: ArrayBuffer = buffers[index];

        // Refused before it is held, as a sector's pack is refused before it is viewed.
        if (buffer.byteLength === description.description.bufferLength) {
          batch.models.push({ buffer, description });
        } else {
          batch.failures.push({
            name: description.name,
            reason:
              `Its pack is ${buffer.byteLength} bytes but its description covers ` +
              `${description.description.bufferLength}, so the two came from different packs`,
          });
        }
      });

      return batch;
    } catch (error: unknown) {
      const reason: string = transformError(error).message;

      return { failures: names.map((name: string) => ({ name, reason })), models: [] };
    }
  }
}
