import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { visualsRawCommands } from "@/core/ipc/commands/visuals-raw";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { ILoadableBump, IVisualBumpStatus, IVisualBumpTextures, toLoadableBumps } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { IVisualSurface, toAlphaTexturePaths } from "@/core/visuals/lib/visual-surface";
import {
  createDdsTexture,
  createDecodedTexture,
  EVisualTextureState,
  ILoadableTexture,
  IVisualTextureStatus,
  toInitialTextureState,
  toLoadableTextures,
} from "@/core/visuals/lib/visual-texture";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, TFlow } from "@/lib/mobx";
import { Nullable, Optional } from "@/lib/types/general";

/** One file's bytes as read, or the reason there are none. Decoding happens later, and only if still wanted. */
interface IVisualTextureRead {
  bytes: Nullable<ArrayBuffer>;
  reason: Nullable<string>;
}

/** One half of a bump pair after upload: the texture when it made it, and what to report either way. */
interface IVisualBumpHalf {
  texture: Nullable<Texture>;
  state: EVisualTextureState;
  reason: Nullable<string>;
}

/**
 * Owns a visual's prepared textures, per-submesh outcomes, and disposal.
 */
export class VisualTextureSet {
  /**
   * Prepares textures inside the caller's cancellable flow.
   *
   * Delegate with `yield*`: cancellation during reads skips texture creation, while cancellation during fallback
   * decoding releases its late results. A successful return transfers disposal to the caller without another yield.
   *
   * @param selected - Description whose resolved roots and logical paths address the reads.
   * @param surfaces - Material state per submesh, deciding which base textures must retain alpha.
   * @returns The prepared set, to publish with geometry and dispose when replaced.
   */
  public static *load(
    selected: SelectedVisualDescription,
    surfaces: ReadonlyMap<number, IVisualSurface>
  ): TFlow<VisualTextureSet> {
    const timer: Timer = new Timer();
    const loaded: VisualTextureSet = new VisualTextureSet();
    const reads: Map<string, IVisualTextureRead> = yield* call(loaded.readTextureFiles(selected));

    loaded.uploadTextures(selected, surfaces, reads);

    loaded.log.info(`Loaded ${reads.size} texture files in:`, formatDuration(timer.lap()));

    // Only base textures the renderer declined use the fallback; bump pairs must keep their packed values and mips.
    const declined: Array<number> = [...loaded.statuses.values()]
      .filter((status) => status.state === EVisualTextureState.UNSUPPORTED_FORMAT)
      .map((status) => status.submeshIndex);

    let transferred: boolean = false;
    let decoding: Promise<void> = Promise.resolve();

    try {
      if (declined.length) {
        decoding = loaded.decodeTextures(selected, declined);

        yield* call(decoding);

        loaded.log.info(
          `Processed texture fallbacks for ${declined.length} submeshes in:`,
          formatDuration(timer.lap())
        );
      }

      transferred = true;

      return loaded;
    } finally {
      if (!transferred) {
        // Decoding can still create textures after cancellation. Release only after every fallback has settled.
        void decoding.then(
          () => loaded.dispose(),
          () => loaded.dispose()
        );
      }
    }
  }

  private readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly textureMap: Map<number, Texture> = new Map();
  private readonly textureStatusMap: Map<number, IVisualTextureStatus> = new Map();
  private readonly bumpMap: Map<number, IVisualBumpTextures> = new Map();
  private readonly bumpStatusMap: Map<number, IVisualBumpStatus> = new Map();

  /**
   * @returns Prepared base textures by submesh index; submeshes can share one texture.
   */
  public get textures(): ReadonlyMap<number, Texture> {
    return this.textureMap;
  }

  /**
   * @returns What became of each submesh's base texture.
   */
  public get statuses(): ReadonlyMap<number, IVisualTextureStatus> {
    return this.textureStatusMap;
  }

  /**
   * @returns Complete bump pairs by submesh index, including resolved dummy pairs.
   */
  public get bumps(): ReadonlyMap<number, IVisualBumpTextures> {
    return this.bumpMap;
  }

  /**
   * @returns What became of each submesh's bump inputs, including incomplete pairs.
   */
  public get bumpStatuses(): ReadonlyMap<number, IVisualBumpStatus> {
    return this.bumpStatusMap;
  }

  private constructor() {}

  /**
   * Releases each distinct texture once after the owner stops presenting this set.
   */
  public dispose(): void {
    const uploaded: Set<Texture> = new Set(this.textureMap.values());

    for (const pair of this.bumpMap.values()) {
      uploaded.add(pair.bump);
      uploaded.add(pair.companion);
    }

    for (const texture of uploaded) {
      texture.dispose();
    }
  }

  /**
   * Read every located texture file of a visual, in parallel, without decoding any of them.
   *
   * Once per **file** rather than once per submesh that names it: a model whose submeshes share a texture used to read
   * and upload it once each, which costs an archive read and a gpu upload for a file already in hand.
   *
   * A failure is a returned reason rather than a throw: one texture that cannot be read is a submesh drawn plain, not
   * a model that fails to open.
   *
   * @param selected - Visual whose textures should be read.
   * @returns Each distinct file's bytes, or the reason there are none, by logical path.
   */
  private async readTextureFiles(selected: SelectedVisualDescription): Promise<Map<string, IVisualTextureRead>> {
    // Base textures and bump pairs in one pass: a dummy pair shared by every degraded submesh is one read either way.
    const paths: Array<string> = [
      ...new Set([
        ...toLoadableTextures(selected.dependencies.textures).map((it: ILoadableTexture) => it.logicalPath),
        ...toLoadableBumps(selected.dependencies.textures, selected.materials).flatMap((it: ILoadableBump) => [
          it.bump,
          it.companion,
        ]),
      ]),
    ];

    if (!paths.length) {
      return new Map();
    }

    this.log.info(`Reading ${paths.length} textures for:`, describeVisualSource(selected.source));

    const reads: Array<[string, IVisualTextureRead]> = await Promise.all(
      paths.map(async (logicalPath: string): Promise<[string, IVisualTextureRead]> => {
        try {
          // Read by the logical path the open already resolved, so the bytes come from the file the description named
          // - a substituted dummy included - rather than from a second lookup that could answer differently.
          const bytes: ArrayBuffer = await assetsRawCommands.readAsset(selected.roots, logicalPath);

          return [logicalPath, { bytes, reason: null }];
        } catch (error: unknown) {
          const transformed: Error = transformError(error);

          this.log.error(`Failed to load texture '${logicalPath}':`, transformed);

          return [logicalPath, { bytes: null, reason: transformed.message }];
        }
      })
    );

    return new Map(reads);
  }

  /**
   * Decode and upload what was read, and say what became of every submesh's reference.
   *
   * @param selected - Visual the textures belong to.
   * @param surfaces - Material state per submesh index, which decides whether a file's alpha has to survive upload.
   * @param reads - What each texture read produced.
   */
  private uploadTextures(
    selected: SelectedVisualDescription,
    surfaces: ReadonlyMap<number, IVisualSurface>,
    reads: Map<string, IVisualTextureRead>
  ): void {
    for (const texture of selected.dependencies.textures) {
      this.textureStatusMap.set(texture.submeshIndex, {
        reason: null,
        state: toInitialTextureState(texture.resolution),
        submeshIndex: texture.submeshIndex,
      });
    }

    // One upload per file, shared by every submesh naming it.
    const uploads: Map<string, Nullable<Texture>> = new Map();
    // Per file rather than per submesh, because the upload is: a DXT1 file drawn by a cut-out surface has to keep the
    // alpha bit its blocks carry, and one upload serves every submesh naming it.
    const alpha: ReadonlySet<string> = toAlphaTexturePaths(surfaces, selected.dependencies.textures);

    for (const { submeshIndex, logicalPath } of toLoadableTextures(selected.dependencies.textures)) {
      const read: Optional<IVisualTextureRead> = reads.get(logicalPath);

      if (!read || read.bytes === null) {
        this.textureStatusMap.set(submeshIndex, {
          reason: read?.reason ?? null,
          state: EVisualTextureState.FAILED,
          submeshIndex,
        });

        continue;
      }

      if (!uploads.has(logicalPath)) {
        uploads.set(logicalPath, createDdsTexture(read.bytes, alpha.has(logicalPath)));
      }

      const uploaded: Nullable<Texture> = uploads.get(logicalPath) ?? null;

      if (uploaded) {
        this.textureMap.set(submeshIndex, uploaded);
      }

      this.textureStatusMap.set(submeshIndex, {
        reason: null,
        state: uploaded ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
        submeshIndex,
      });
    }

    this.uploadBumps(selected, reads, uploads);
  }

  /**
   * Upload every complete bump pair, sharing uploads with the base textures and between submeshes.
   *
   * A pair lands only when both halves uploaded: the engine samples both every texel, so half a pair shades nothing,
   * while each half still reports its own outcome so the panel can say which one failed. The renderer's own loader is
   * the only decoder here; a bump in a layout it refuses stays unshaded rather than going through the png fallback,
   * whose single image would lose the mip chain a bump relies on at distance.
   *
   * @param selected - Visual the materials belong to.
   * @param reads - What each texture read produced.
   * @param uploads - Uploads so far by logical path, shared so a file read once is uploaded once.
   */
  private uploadBumps(
    selected: SelectedVisualDescription,
    reads: Map<string, IVisualTextureRead>,
    uploads: Map<string, Nullable<Texture>>
  ): void {
    for (const loadable of toLoadableBumps(selected.dependencies.textures, selected.materials)) {
      const bump: IVisualBumpHalf = this.uploadBumpHalf(loadable.bump, reads, uploads);
      const companion: IVisualBumpHalf = this.uploadBumpHalf(loadable.companion, reads, uploads);

      if (bump.texture && companion.texture) {
        this.bumpMap.set(loadable.submeshIndex, { bump: bump.texture, companion: companion.texture });
      }

      this.bumpStatusMap.set(loadable.submeshIndex, {
        submeshIndex: loadable.submeshIndex,
        bump: bump.state,
        companion: companion.state,
        reason: bump.reason ?? companion.reason,
      });
    }
  }

  /**
   * Upload one half of a pair, or say why it is not on the gpu.
   *
   * @param logicalPath - The located file.
   * @param reads - What each texture read produced.
   * @param uploads - Uploads so far by logical path, shared so a file read once is uploaded once.
   * @returns The texture when it uploaded, and the state and reason either way.
   */
  private uploadBumpHalf(
    logicalPath: string,
    reads: Map<string, IVisualTextureRead>,
    uploads: Map<string, Nullable<Texture>>
  ): IVisualBumpHalf {
    const read: Optional<IVisualTextureRead> = reads.get(logicalPath);

    if (!read || read.bytes === null) {
      return { texture: null, state: EVisualTextureState.FAILED, reason: read?.reason ?? null };
    }

    if (!uploads.has(logicalPath)) {
      uploads.set(logicalPath, createDdsTexture(read.bytes));
    }

    const texture: Nullable<Texture> = uploads.get(logicalPath) ?? null;

    return {
      texture,
      state: texture ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
      reason: null,
    };
  }

  /**
   * Decode each file three.js declined once, sharing the fallback texture between submeshes naming it.
   *
   * A successful fallback is reported as decoded because it has no mip chain. A refused fallback leaves the original
   * unsupported status for the materials panel to report.
   *
   * @param selected - Visual the textures belong to, whose roots address the read.
   * @param declined - Submesh indices whose texture the renderer's own loader refused.
   */
  private async decodeTextures(selected: SelectedVisualDescription, declined: Array<number>): Promise<void> {
    const declinedIndices: Set<number> = new Set(declined);
    const submeshesByPath: Map<string, Array<number>> = new Map();

    for (const { submeshIndex, logicalPath } of toLoadableTextures(selected.dependencies.textures)) {
      if (!declinedIndices.has(submeshIndex)) {
        continue;
      }

      const submeshes: Array<number> = submeshesByPath.get(logicalPath) ?? [];

      submeshes.push(submeshIndex);
      submeshesByPath.set(logicalPath, submeshes);
    }

    await Promise.all(
      Array.from(submeshesByPath, async ([logicalPath, submeshes]) => {
        try {
          const png: ArrayBuffer = await visualsRawCommands.readTexture(selected.roots, logicalPath);
          const texture: Texture = await createDecodedTexture(png);

          for (const submeshIndex of submeshes) {
            this.textureMap.set(submeshIndex, texture);
            this.textureStatusMap.set(submeshIndex, {
              reason: null,
              state: EVisualTextureState.DECODED,
              submeshIndex,
            });
          }
        } catch (error: unknown) {
          // Left as unsupported rather than failed: nothing broke, the format simply decodes nowhere.
          this.log.info(`Texture '${logicalPath}' decodes nowhere:`, transformError(error).message);
        }
      })
    );
  }
}
