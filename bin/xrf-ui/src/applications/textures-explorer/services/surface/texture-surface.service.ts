import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { SRGBColorSpace, Texture } from "three";

import {
  EMPTY_TEXTURE_SURFACE,
  ITextureBumpAssets,
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
import { createDdsTexture, createDecodedTexture } from "@/core/visuals/lib/visual-texture";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The textures the lit surface is drawn from, uploaded to the gpu.
 */
@Injectable()
export class TextureSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public textures: Loadable<ITextureSurfaceTextures> = createLoadable(EMPTY_TEXTURE_SURFACE);

  /**
   * Which texture the uploads above belong to, once one has been attempted for it.
   */
  @Observable()
  public uploaded: Nullable<string> = null;

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
    disposeTextures(listTextureSurfaceTextures(this.textures.value));

    this.textures = this.textures.asLoading(EMPTY_TEXTURE_SURFACE);
    this.uploaded = null;

    const uploads: Array<Promise<Nullable<Texture>>> = [];
    let published: Nullable<ITextureSurfaceTextures> = null;

    try {
      const { roots } = description;
      const bumpAssets: Nullable<ITextureBumpAssets> = selectTextureBumpAssets(description);

      const base: Nullable<Texture> = description.texture
        ? yield* call(this.read(uploads, roots, description.texture.logicalPath, true))
        : null;

      // Colour, not data: a base texture holds sRGB values, and saying so is what makes the unlit body match the flat
      // picture of the same file. The pair is left alone on purpose - a packed normal is numbers, and linearising it
      // would move every one of them.
      if (base) {
        base.colorSpace = SRGBColorSpace;
      }

      const bump: Nullable<Texture> = bumpAssets
        ? yield* call(this.read(uploads, roots, bumpAssets.bump.logicalPath, false))
        : null;
      const companion: Nullable<Texture> = bumpAssets
        ? yield* call(this.read(uploads, roots, bumpAssets.companion.logicalPath, false))
        : null;

      published = {
        aspect: toTextureAspect(description),
        base,
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump, companion } : null,
      };

      this.textures = this.textures.asReady(published);
      this.uploaded = description.reference;
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
    disposeTextures(listTextureSurfaceTextures(this.textures.value));

    runInAction(() => {
      this.textures = createLoadable(EMPTY_TEXTURE_SURFACE);
      this.uploaded = null;
    });
  }

  /**
   * Starts one read and records it, so the run that asked for it can release it whatever happens next.
   *
   * @param uploads - Where the read is recorded.
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @param isDecodable - Whether a layout three.js refuses may be decoded by the backend instead.
   * @returns The upload in progress.
   */
  private read(
    uploads: Array<Promise<Nullable<Texture>>>,
    roots: XrayRoots,
    logicalPath: string,
    isDecodable: boolean
  ): Promise<Nullable<Texture>> {
    const upload: Promise<Nullable<Texture>> = this.upload(roots, logicalPath, isDecodable);

    uploads.push(upload);

    return upload;
  }

  /**
   * Reads one file and uploads it.
   *
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file.
   * @param isDecodable - Whether a layout three.js refuses may be decoded by the backend instead.
   * @returns The texture, or null when nothing could be uploaded for it.
   */
  private async upload(roots: XrayRoots, logicalPath: string, isDecodable: boolean): Promise<Nullable<Texture>> {
    try {
      const compressed: Nullable<Texture> = createDdsTexture(await assetsRawCommands.readAsset(roots, logicalPath));

      if (compressed || !isDecodable) {
        return compressed;
      }

      return await createDecodedTexture(await texturesRawCommands.readTexture(roots, logicalPath));
    } catch (error: unknown) {
      this.log.error(`Failed to upload '${logicalPath}':`, error);

      return null;
    }
  }
}

/**
 * Releases the gpu resources of whatever was uploaded.
 *
 * @param textures - Uploads to release, present or not.
 */
function disposeTextures(textures: ReadonlyArray<Texture>): void {
  for (const texture of textures) {
    texture.dispose();
  }
}
