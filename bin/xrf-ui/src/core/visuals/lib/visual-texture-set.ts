import { Nullable, Optional } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { visualsRawCommands } from "@/core/ipc/commands/visuals-raw";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { readDdsFile } from "@/core/render/lib/dds";
import { IRenderSurface } from "@/core/render/lib/surface/render-surface";
import { ILoadableBump, IVisualBumpFiles, IVisualBumpStatus, toLoadableBumps } from "@/core/visuals/lib/visual-bump";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { toAlphaTexturePaths } from "@/core/visuals/lib/visual-surface";
import {
  EVisualTextureState,
  ILoadableTexture,
  IVisualTextureFile,
  IVisualTextureStatus,
  toInitialTextureState,
  toLoadableTextures,
} from "@/core/visuals/lib/visual-texture";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, TFlow } from "@/lib/mobx";

/** One file's bytes as read, or the reason there are none. Decoding happens later, and only if still wanted. */
interface IVisualTextureRead {
  bytes: Nullable<ArrayBuffer>;
  reason: Nullable<string>;
}

/** One half of a bump pair once read: the file when three.js will take it, and what to report either way. */
interface IVisualBumpHalf {
  file: Nullable<IVisualTextureFile>;
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
   * @param selected - Description whose resolved roots and logical paths address the reads.
   * @param surfaces - Material state per submesh, deciding which base textures must retain alpha.
   * @returns What was read, to publish with the geometry it belongs to.
   */
  public static *load(
    selected: SelectedVisualDescription,
    surfaces: ReadonlyMap<number, IRenderSurface>
  ): TFlow<VisualTextureSet> {
    const timer: Timer = new Timer();
    const loaded: VisualTextureSet = new VisualTextureSet();
    const reads: Map<string, IVisualTextureRead> = yield* call(loaded.readTextureFiles(selected));

    loaded.resolveTextures(selected, surfaces, reads);

    loaded.log.info(`Loaded ${reads.size} texture files in:`, formatDuration(timer.lap()));

    // Only base textures the renderer declined use the fallback; bump pairs must keep their packed values and mips.
    const declined: Array<number> = [...loaded.statuses.values()]
      .filter((status) => status.state === EVisualTextureState.UNSUPPORTED_FORMAT)
      .map((status) => status.submeshIndex);

    if (declined.length) {
      yield* call(loaded.decodeTextures(selected, declined));

      loaded.log.info(`Processed texture fallbacks for ${declined.length} submeshes in:`, formatDuration(timer.lap()));
    }

    return loaded;
  }

  private readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly textureMap: Map<number, IVisualTextureFile> = new Map();
  private readonly textureStatusMap: Map<number, IVisualTextureStatus> = new Map();
  private readonly bumpMap: Map<number, IVisualBumpFiles> = new Map();
  private readonly bumpStatusMap: Map<number, IVisualBumpStatus> = new Map();

  /**
   * @returns Base files by submesh index; submeshes can share one file.
   */
  public get textures(): ReadonlyMap<number, IVisualTextureFile> {
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
  public get bumps(): ReadonlyMap<number, IVisualBumpFiles> {
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
   * Read every located texture file of a visual, in parallel, without decoding any of them.
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
   * Decide what each submesh is drawn from, and say what became of its reference.
   *
   * @param selected - Visual the textures belong to.
   * @param surfaces - Material state per submesh index, which decides whether a file's alpha has to survive upload.
   * @param reads - What each texture read produced.
   */
  private resolveTextures(
    selected: SelectedVisualDescription,
    surfaces: ReadonlyMap<number, IRenderSurface>,
    reads: Map<string, IVisualTextureRead>
  ): void {
    for (const texture of selected.dependencies.textures) {
      this.textureStatusMap.set(texture.submeshIndex, {
        reason: null,
        state: toInitialTextureState(texture.resolution),
        submeshIndex: texture.submeshIndex,
      });
    }

    // One answer per file, shared by every submesh naming it: whether three.js takes a layout is a property
    // of the file, and asking twice would say the same thing twice.
    const taken: Map<string, Nullable<IVisualTextureFile>> = new Map();
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

      if (!taken.has(logicalPath)) {
        const isAlphaRead: boolean = alpha.has(logicalPath);

        // A base texture is a picture, so it is read as sRGB where it is uploaded; whether its alpha survives
        // is the surface's answer, and it travels with the file because only this side knows the surfaces.
        taken.set(
          logicalPath,
          readDdsFile(read.bytes, isAlphaRead).file
            ? { bytes: read.bytes, isAlphaRead, isDecoded: false, logicalPath }
            : null
        );
      }

      const file: Nullable<IVisualTextureFile> = taken.get(logicalPath) ?? null;

      if (file) {
        this.textureMap.set(submeshIndex, file);
      }

      this.textureStatusMap.set(submeshIndex, {
        reason: null,
        state: file ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
        submeshIndex,
      });
    }

    this.resolveBumps(selected, reads, taken);
  }

  /**
   * Resolve every complete bump pair, sharing answers with the base textures and between submeshes.
   *
   * @param selected - Visual the materials belong to.
   * @param reads - What each texture read produced.
   * @param taken - What was decided per file so far, shared so a file read once is decided once.
   */
  private resolveBumps(
    selected: SelectedVisualDescription,
    reads: Map<string, IVisualTextureRead>,
    taken: Map<string, Nullable<IVisualTextureFile>>
  ): void {
    for (const loadable of toLoadableBumps(selected.dependencies.textures, selected.materials)) {
      const bump: IVisualBumpHalf = this.resolveBumpHalf(loadable.bump, reads, taken);
      const companion: IVisualBumpHalf = this.resolveBumpHalf(loadable.companion, reads, taken);

      if (bump.file && companion.file) {
        this.bumpMap.set(loadable.submeshIndex, { bump: bump.file, companion: companion.file });
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
   * Resolve one half of a pair, or say why nothing will draw it.
   *
   * @param logicalPath - The located file.
   * @param reads - What each texture read produced.
   * @param taken - What was decided per file so far, shared so a file read once is decided once.
   * @returns The file when three.js will take it, and the state and reason either way.
   */
  private resolveBumpHalf(
    logicalPath: string,
    reads: Map<string, IVisualTextureRead>,
    taken: Map<string, Nullable<IVisualTextureFile>>
  ): IVisualBumpHalf {
    const read: Optional<IVisualTextureRead> = reads.get(logicalPath);

    if (!read || read.bytes === null) {
      return { file: null, reason: read?.reason ?? null, state: EVisualTextureState.FAILED };
    }

    if (!taken.has(logicalPath)) {
      // No colour decode: a bump pair's channels are a packed normal, and decoding one would bend every vector
      // in it. The file says which answer it wants, because the side that uploads cannot know.
      taken.set(
        logicalPath,
        readDdsFile(read.bytes).file ? { bytes: read.bytes, isAlphaRead: false, isDecoded: false, logicalPath } : null
      );
    }

    const file: Nullable<IVisualTextureFile> = taken.get(logicalPath) ?? null;

    return {
      file,
      reason: null,
      state: file ? EVisualTextureState.APPLIED : EVisualTextureState.UNSUPPORTED_FORMAT,
    };
  }

  /**
   * Ask the backend for a picture of each file three.js declined, shared between the submeshes naming it.
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
          const file: IVisualTextureFile = { bytes: png, isAlphaRead: false, isDecoded: true, logicalPath };

          for (const submeshIndex of submeshes) {
            this.textureMap.set(submeshIndex, file);
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
