import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { IDdsRead, readDdsFile } from "@/core/render/lib/dds";
import { Logger } from "@/lib/logging";

/**
 * Reads the files a level's textures come from.
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

  /** Forgets the level, so a read after it closes finds nothing rather than the last one's files. */
  public close(): void {
    this.paths = new Map();
    this.roots = null;
  }

  /**
   * Reads one reference's file.
   *
   * @param request - The reference, and what the surfaces drawn with it need of it.
   * @returns Its bytes and how to sample them, or the reason there are none.
   */
  public async read(request: ISectorTextureRequest): Promise<ILevelTextureDelivery> {
    const reference: string = request.reference;
    const logicalPath: Maybe<string> = this.paths.get(reference);

    if (!this.roots || !logicalPath) {
      return this.toFailure(request, `Nothing in the mounted roots answers to '${reference}'`);
    }

    try {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(this.roots, logicalPath);
      // Read here rather than where it is uploaded, because whether the reader models the layout is a question
      // about the file and not about the graphics context. Deciding it on the far side would mean asking back.
      const read: IDdsRead = readDdsFile(bytes, request.isAlphaRead);

      if (read.file) {
        return {
          bytes,
          isAlphaRead: request.isAlphaRead,
          isDecoded: false,
          isMipped: request.isMipped,
          reason: null,
          reference,
        };
      }

      // A layout the reader does not model; the backend expands those to a picture instead.
      this.log.info(`Texture '${reference}' is decoded rather than read as it is:`, read.refusal);

      return {
        bytes: await texturesRawCommands.readTexture(this.roots, logicalPath),
        isAlphaRead: request.isAlphaRead,
        isDecoded: true,
        isMipped: request.isMipped,
        reason: null,
        reference,
      };
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(`Failed to read level texture '${reference}':`, transformed);

      return this.toFailure(request, transformed.message);
    }
  }

  private toFailure(request: ISectorTextureRequest, reason: string): ILevelTextureDelivery {
    return {
      bytes: new ArrayBuffer(0),
      isAlphaRead: request.isAlphaRead,
      isDecoded: false,
      isMipped: request.isMipped,
      reason,
      reference: request.reference,
    };
  }
}
