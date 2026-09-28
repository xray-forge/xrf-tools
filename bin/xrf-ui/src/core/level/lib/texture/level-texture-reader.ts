import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { IBulkRequest, requestBulk } from "@/core/ipc/bulk";
import { assetsBulkRoutes } from "@/core/ipc/commands/assets-bulk";
import { texturesBulkRoutes } from "@/core/ipc/commands/textures-bulk";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { Logger } from "@/lib/logging";

/**
 * Says where a level's textures are fetched from: the renderer fetches them itself, so no byte of them crosses the
 * page or the window's thread.
 */
export class LevelTextureReader {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Where each reference resolved to at open, so a read is a read rather than a second search. */
  private paths: ReadonlyMap<string, string> = new Map();

  private roots: Nullable<XrayRoots> = null;

  /**
   * Takes the level a later read reads against.
   *
   * @param roots - Roots the level was opened in.
   * @param references - What each texture reference came to, from the open.
   */
  public open(roots: XrayRoots, references: ReadonlyArray<LevelTextureReference>): void {
    this.roots = roots;
    this.paths = new Map(
      references
        .filter((it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } =>
          Boolean(it.logicalPath)
        )
        .map((it) => [it.reference, it.logicalPath])
    );
  }

  /**
   * Adds what more references came to, read beside the open: what the level's grass, lights and spawned models bind.
   *
   * @param references - What each texture reference came to.
   */
  public add(references: ReadonlyArray<LevelTextureReference>): void {
    const paths: Map<string, string> = new Map(this.paths);

    for (const it of references) {
      if (it.logicalPath) {
        paths.set(it.reference, it.logicalPath);
      }
    }

    this.paths = paths;
  }

  /** Forgets the level, so a read after it closes finds nothing rather than the last one's files. */
  public close(): void {
    this.paths = new Map();
    this.roots = null;
  }

  /**
   * Says where one reference's file is fetched from.
   *
   * @param request - The reference, and what the surfaces drawn with it need of it.
   * @returns The requests fetching its file and the backend's picture of it, or the reason there are none.
   */
  public async read(request: ISectorTextureRequest): Promise<ILevelTextureDelivery> {
    const reference: string = request.reference;
    const logicalPath: Maybe<string> = this.paths.get(reference);

    if (!this.roots || !logicalPath) {
      return LevelTextureReader.toFailure(reference, `Nothing in the mounted roots answers to '${reference}'`);
    }

    try {
      // Both asked for now, though the picture is fetched only where the renderer's reader refuses the file: whether
      // it does is a question about the file, answered where the file is read.
      const [file, picture]: [IBulkRequest, IBulkRequest] = await Promise.all([
        requestBulk(assetsBulkRoutes.readAsset(this.roots, logicalPath)),
        requestBulk(texturesBulkRoutes.readTexture(this.roots, logicalPath)),
      ]);

      return { reason: null, reference, requests: { file, picture } };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to ask for level texture '${reference}':`, transformed);

      return LevelTextureReader.toFailure(reference, transformed.message);
    }
  }

  private static toFailure(reference: string, reason: string): ILevelTextureDelivery {
    return { reason, reference, requests: null };
  }
}
