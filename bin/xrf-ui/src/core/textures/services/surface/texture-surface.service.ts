import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { transformError } from "@/core/error/lib";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { texturesRawCommands } from "@/core/ipc/commands/textures-raw";
import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { IRenderTextureTexels, readDdsTexels } from "@/core/render/lib/render-texels";
import { createDdsTexture, createDecodedTexture, IRenderTextureUpload } from "@/core/render/lib/render-texture";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureBumpAssets,
  ITextureBumpTexels,
  ITextureSurfaceTextures,
  listTextureSurfaceTextures,
  selectTextureBumpAssets,
  toTextureAspect,
} from "@/core/textures/lib/texture-surface";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** One half of the pair as it arrived: on the gpu always, and on the cpu when its layout stores texels plainly. */
interface ITextureBumpHalf {
  texture: Texture;
  texels: Nullable<IRenderTextureTexels>;
}

/**
 * The textures the lit surface is drawn from, uploaded to the gpu.
 */
@Injectable()
export class TextureSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public textures: AsyncState<ITextureSurfaceTextures> = AsyncState.idle(EMPTY_TEXTURE_SURFACE);

  /**
   * Which texture the uploads above belong to, once one has been attempted for it.
   */
  @Observable()
  public uploaded: Nullable<string> = null;

  /**
   * The pair's texels on the cpu, when its layout stores them plainly.
   */
  @Observable()
  public bumpTexels: Nullable<ITextureBumpTexels> = null;

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Uploads what the surface draws for one texture.
   *
   * @param description - The texture as the backend resolved it.
   */
  @LatestFlow("textures")
  public *load(description: TextureDescription): TFlow {
    const timer: Timer = new Timer();

    this.disposeTextures(listTextureSurfaceTextures(this.textures.value));

    this.textures = this.textures.asLoading(EMPTY_TEXTURE_SURFACE);
    this.uploaded = null;
    this.bumpTexels = null;

    const uploads: Array<Promise<Nullable<Texture>>> = [];
    let published: Nullable<ITextureSurfaceTextures> = null;

    try {
      const { roots } = description;
      const bumpAssets: Nullable<ITextureBumpAssets> = selectTextureBumpAssets(description);

      const base: Nullable<Texture> = description.texture
        ? yield* call(this.readBase(uploads, roots, description.texture.logicalPath))
        : null;

      const bump: Nullable<ITextureBumpHalf> = bumpAssets
        ? yield* call(this.readBumpHalf(uploads, roots, bumpAssets.bump.logicalPath))
        : null;
      const companion: Nullable<ITextureBumpHalf> =
        bump && bumpAssets ? yield* call(this.readBumpHalf(uploads, roots, bumpAssets.companion.logicalPath)) : null;

      if (bump && !companion) {
        bump.texture.dispose();
      }

      published = {
        aspect: toTextureAspect(description),
        base,
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump: bump.texture, companion: companion.texture } : null,
      };

      this.textures = this.textures.asReady(published);
      this.uploaded = description.reference;
      this.bumpTexels = bump?.texels && companion?.texels ? { bump: bump.texels, companion: companion.texels } : null;

      this.log.info(
        "Texture surface loaded:",
        description.reference,
        { base: Boolean(base), bump: Boolean(bump), companion: Boolean(companion) },
        "in",
        formatDuration(timer.elapsed())
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to upload the texture surface:",
        description.reference,
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.textures = this.textures.asFailed(transformed, EMPTY_TEXTURE_SURFACE);
      this.uploaded = description.reference;
    } finally {
      // Reached on cancellation too, since abandoning a flow returns through it. Released through the promises rather
      // than through the textures, because a read the next selection cancelled is still in flight here and will upload
      // its texture after this line: whatever this run does not publish is gpu memory no surface will ever draw.
      if (!published) {
        for (const upload of uploads) {
          void upload.then((texture: Nullable<Texture>) => texture?.dispose());
        }
      }
    }
  }

  /**
   * Drops whatever is uploaded, for a session that is ending or a texture that is no longer selected.
   */
  public clear(): void {
    cancelFlow(this, "textures");
    this.disposeTextures(listTextureSurfaceTextures(this.textures.value));

    runInAction(() => {
      this.textures = this.textures.asIdle(EMPTY_TEXTURE_SURFACE);
      this.uploaded = null;
      this.bumpTexels = null;
    });
  }

  /**
   * Reads the base texture, falling back to the backend's decode for a layout three.js refuses.
   *
   * @param uploads - Where the read is recorded, so a run that never publishes can still release it.
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @returns The upload in progress.
   */
  private readBase(
    uploads: Array<Promise<Nullable<Texture>>>,
    roots: XrayRoots,
    logicalPath: string
  ): Promise<Nullable<Texture>> {
    const upload: Promise<Nullable<Texture>> = this.guard(logicalPath, async () => {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(roots, logicalPath);

      // Colour, not data: a base texture holds sRGB values, and saying so is what makes the unlit body match the flat
      // picture of the same file. Said at the upload so both paths agree, rather than patched onto whichever wins.
      const upload: IRenderTextureUpload = createDdsTexture(bytes, { isColor: true });

      return (
        upload.texture ??
        (await createDecodedTexture(await texturesRawCommands.readTexture(roots, logicalPath), { isColor: true }))
      );
    });

    uploads.push(upload);

    return upload;
  }

  /**
   * Reads one half of the pair, and its texels where the layout stores them plainly.
   *
   * @param uploads - Where the read is recorded, so a run that never publishes can still release it.
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @returns The upload in progress.
   */
  private readBumpHalf(
    uploads: Array<Promise<Nullable<Texture>>>,
    roots: XrayRoots,
    logicalPath: string
  ): Promise<Nullable<ITextureBumpHalf>> {
    const half: Promise<Nullable<ITextureBumpHalf>> = this.guard(logicalPath, async () => {
      const bytes: ArrayBuffer = await assetsRawCommands.readAsset(roots, logicalPath);
      const upload: IRenderTextureUpload = createDdsTexture(bytes);

      return upload.texture ? { texels: readDdsTexels(bytes), texture: upload.texture } : null;
    });

    uploads.push(half.then((it: Nullable<ITextureBumpHalf>) => it?.texture ?? null));

    return half;
  }

  /**
   * Runs one read, reporting a failure as an absent answer rather than as a thrown one.
   *
   * @param logicalPath - Engine identity of the file, for the log line.
   * @param read - The read to run.
   * @returns What the read answered, or null when it failed.
   */
  private async guard<T>(logicalPath: string, read: () => Promise<Nullable<T>>): Promise<Nullable<T>> {
    try {
      return await read();
    } catch (error: unknown) {
      this.log.error(`Failed to upload '${logicalPath}':`, error);

      return null;
    }
  }

  /**
   * Releases the gpu resources of whatever was uploaded.
   *
   * @param textures - Uploads to release, present or not.
   */
  private disposeTextures(textures: ReadonlyArray<Texture>): void {
    for (const texture of textures) {
      texture.dispose();
    }
  }
}
