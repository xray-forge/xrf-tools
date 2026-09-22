import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, RefObservable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { IDdsRead, readDdsFile } from "@/core/render/lib/dds";
import { IRenderTextureTexels, readDdsTexels } from "@/core/render/lib/texture/render-texels";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureBumpAssets,
  ITextureBumpTexels,
  ITextureSurfaceFile,
  ITextureSurfaceFiles,
  selectTextureBumpAssets,
  toTextureAspect,
} from "@/core/textures/lib/texture-surface";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** One half of the pair as it was read: the file always, and its texels when the layout stores them plainly. */
interface ITextureBumpHalf {
  file: ITextureSurfaceFile;
  texels: Nullable<IRenderTextureTexels>;
}

/**
 * The files the lit surface is drawn from.
 */
@Injectable()
export class TextureSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @RefObservable()
  public files: AsyncState<ITextureSurfaceFiles> = AsyncState.idle(EMPTY_TEXTURE_SURFACE);

  /**
   * Which texture the files above belong to, once a read has been attempted for it.
   */
  @Observable()
  public reference: Nullable<string> = null;

  /**
   * The pair's texels on the cpu, when its layout stores them plainly.
   */
  @RefObservable()
  public bumpTexels: Nullable<ITextureBumpTexels> = null;

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Reads what the surface draws for one texture.
   *
   * @param description - The texture as the backend resolved it.
   */
  @LatestFlow("files")
  public *load(description: TextureDescription): TFlow {
    const timer: Timer = new Timer();

    this.files = this.files.asLoading(EMPTY_TEXTURE_SURFACE);
    this.reference = null;
    this.bumpTexels = null;

    try {
      const { roots } = description;
      const bumpAssets: Nullable<ITextureBumpAssets> = selectTextureBumpAssets(description);

      const base: Nullable<ITextureSurfaceFile> = description.texture
        ? yield* call(this.readBase(roots, description.texture.logicalPath))
        : null;

      const bump: Nullable<ITextureBumpHalf> = bumpAssets
        ? yield* call(this.readBumpHalf(roots, bumpAssets.bump.logicalPath))
        : null;
      const companion: Nullable<ITextureBumpHalf> =
        bump && bumpAssets ? yield* call(this.readBumpHalf(roots, bumpAssets.companion.logicalPath)) : null;

      this.files = this.files.asReady({
        aspect: toTextureAspect(description),
        base,
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump: bump.file, companion: companion.file } : null,
      });
      this.reference = description.reference;
      this.bumpTexels = bump?.texels && companion?.texels ? { bump: bump.texels, companion: companion.texels } : null;

      this.log.info(
        "Texture surface read:",
        description.reference,
        { base: Boolean(base), bump: Boolean(bump), companion: Boolean(companion) },
        "in",
        formatDuration(timer.elapsed())
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to read the texture surface:",
        description.reference,
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.files = this.files.asFailed(transformed, EMPTY_TEXTURE_SURFACE);
      this.reference = description.reference;
    }
  }

  /**
   * Drops whatever was read, for a session that is ending or a texture that is no longer selected.
   */
  public clear(): void {
    cancelFlow(this, "files");

    runInAction(() => {
      this.files = this.files.asIdle(EMPTY_TEXTURE_SURFACE);
      this.reference = null;
      this.bumpTexels = null;
    });
  }

  /**
   * Reads the base file, falling back to the backend's decode for a layout three.js refuses.
   *
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @returns The file, or nothing when it could not be read.
   */
  private readBase(roots: XrayRoots, logicalPath: string): Promise<Nullable<ITextureSurfaceFile>> {
    return this.guard(logicalPath, async () => {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(roots, logicalPath);
      const read: IDdsRead = readDdsFile(bytes, true);

      if (read.file) {
        return { bytes, height: read.file.height, isDecoded: false, width: read.file.width };
      }

      this.log.info(`Texture '${logicalPath}' is decoded rather than read as it is:`, read.refusal);

      // Measured by whoever decodes the picture: the png says what it is, and nothing here has to parse it.
      return {
        bytes: await texturesRawCommands.readTexture(roots, logicalPath),
        height: 0,
        isDecoded: true,
        width: 0,
      };
    });
  }

  /**
   * Reads one half of the pair, and its texels where the layout stores them plainly.
   *
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @returns The half, or nothing for a layout no side would draw.
   */
  private readBumpHalf(roots: XrayRoots, logicalPath: string): Promise<Nullable<ITextureBumpHalf>> {
    return this.guard(logicalPath, async () => {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(roots, logicalPath);
      const read: IDdsRead = readDdsFile(bytes);

      // No fallback for a pair: the decode reads its packed values, and a picture of them shades nothing.
      return read.file
        ? {
            file: { bytes, height: read.file.height, isDecoded: false, width: read.file.width },
            texels: readDdsTexels(bytes),
          }
        : null;
    });
  }

  /**
   * Runs one read, reporting a failure as an absent answer rather than as a thrown one.
   *
   * @param logicalPath - Engine identity of the file, for the report.
   * @param read - What to run.
   * @returns What it answered, or null when it failed.
   */
  private async guard<T>(logicalPath: string, read: () => Promise<Nullable<T>>): Promise<Nullable<T>> {
    try {
      return await read();
    } catch (error: unknown) {
      this.log.error(`Failed to read '${logicalPath}':`, transformError(error));

      return null;
    }
  }
}
