import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { SRGBColorSpace, Texture } from "three";

import {
  EMPTY_TEXTURE_SURFACE,
  ITextureBumpAssets,
  ITextureBumpTexels,
  ITextureSurfaceTextures,
  listTextureSurfaceTextures,
  selectTextureBumpAssets,
  toTextureAspect,
} from "@/applications/textures-explorer/lib/texture-surface";
import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { texturesRawCommands } from "@/core/bindings/commands/textures-raw";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import {
  createDdsTexture,
  createDecodedTexture,
  IVisualTextureTexels,
  readDdsTexels,
} from "@/core/visuals/lib/visual-texture";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** One half of the pair as it arrived: on the gpu always, and on the cpu when its layout stores texels plainly. */
interface ITextureBumpHalf {
  texture: Texture;
  texels: Nullable<IVisualTextureTexels>;
}

/**
 * The textures the lit surface is drawn from, uploaded to the gpu.
 */
@Injectable()
export class TextureSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public textures: Loadable<ITextureSurfaceTextures> = Loadable.idle(EMPTY_TEXTURE_SURFACE);

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

      // Colour, not data: a base texture holds sRGB values, and saying so is what makes the unlit body match the flat
      // picture of the same file. The pair is left alone on purpose - a packed normal is numbers, and linearising it
      // would move every one of them.
      if (base) {
        base.colorSpace = SRGBColorSpace;
      }

      const bump: Nullable<ITextureBumpHalf> = bumpAssets
        ? yield* call(this.readBumpHalf(uploads, roots, bumpAssets.bump.logicalPath))
        : null;
      const companion: Nullable<ITextureBumpHalf> = bumpAssets
        ? yield* call(this.readBumpHalf(uploads, roots, bumpAssets.companion.logicalPath))
        : null;

      published = {
        aspect: toTextureAspect(description),
        base,
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump: bump.texture, companion: companion.texture } : null,
      };

      this.textures = this.textures.asReady(published);
      this.uploaded = description.reference;
      this.bumpTexels = bump?.texels && companion?.texels ? { bump: bump.texels, companion: companion.texels } : null;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to upload the texture surface:", transformed);

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
   *
   * Plain rather than a flow, because nothing here waits: the one asynchronous thing in reach is an upload still in
   * flight, and this abandons it rather than joining it.
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

      const compressed: Nullable<Texture> = createDdsTexture(bytes);

      return compressed ?? (await createDecodedTexture(await texturesRawCommands.readTexture(roots, logicalPath)));
    });

    uploads.push(upload);

    return upload;
  }

  /**
   * Reads one half of the pair, and its texels where the layout stores them plainly.
   *
   * Never the backend's png fallback, unlike the base: a packed plane re-encoded through an srgb path would report
   * values it does not hold, and a bump drawn from those is worse than no bump at all.
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
      const texture: Nullable<Texture> = createDdsTexture(bytes);

      return texture ? { texels: readDdsTexels(bytes), texture } : null;
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
