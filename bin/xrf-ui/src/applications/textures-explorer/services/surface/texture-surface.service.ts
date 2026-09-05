import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceTextures } from "@/applications/textures-explorer/lib/texture-surface";
import { getLocatedAsset } from "@/core/assets/lib";
import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { texturesRawCommands } from "@/core/bindings/commands/textures-raw";
import { AssetTextureShape, TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayAsset, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { createDdsTexture, createDecodedTexture } from "@/core/visuals/lib/visual-texture";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
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
    this.release();

    runInAction(() => {
      this.textures = createLoadable(EMPTY_TEXTURE_SURFACE);
      this.uploaded = null;
    });
  }

  /**
   * Uploads what the surface draws for one texture.
   *
   * @param description - The texture as the backend resolved it.
   */
  @LatestFlow("textures")
  public *load(description: TextureDescription): TFlow {
    this.release();

    this.textures = this.textures.asLoading(EMPTY_TEXTURE_SURFACE);
    this.uploaded = null;

    try {
      const { roots } = description;
      const bumpAssets: Nullable<[XrayAsset, XrayAsset]> = selectBumpAssets(description);

      const base: Nullable<Texture> = description.texture
        ? yield* call(this.upload(roots, description.texture.logicalPath, true))
        : null;
      const bump: Nullable<Texture> = bumpAssets
        ? yield* call(this.upload(roots, bumpAssets[0].logicalPath, false))
        : null;
      const companion: Nullable<Texture> = bumpAssets
        ? yield* call(this.upload(roots, bumpAssets[1].logicalPath, false))
        : null;

      this.textures = this.textures.asReady({
        aspect: toTextureAspect(description),
        base,
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump, companion } : null,
      });
      this.uploaded = description.reference;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to upload the texture surface:", transformed);

      this.textures = this.textures.asFailed(transformed, EMPTY_TEXTURE_SURFACE);
      this.uploaded = description.reference;
    }
  }

  /** Drops whatever is uploaded, for a session that is ending or a texture that is no longer selected. */
  @LatestFlow("textures")
  public *clear(): TFlow {
    this.release();

    this.textures = createLoadable(EMPTY_TEXTURE_SURFACE);
    this.uploaded = null;

    yield* call(Promise.resolve());
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
      const uploaded: Nullable<Texture> = createDdsTexture(await assetsRawCommands.readAsset(roots, logicalPath));

      if (uploaded || !isDecodable) {
        return uploaded;
      }

      return await createDecodedTexture(await texturesRawCommands.readTexture(roots, logicalPath));
    } catch (error: unknown) {
      this.log.error(`Failed to upload '${logicalPath}':`, error);

      return null;
    }
  }

  /** Releases the gpu resources of whatever is currently uploaded. */
  private release(): void {
    const current: Nullable<ITextureSurfaceTextures> = this.textures.value;

    for (const texture of [current?.base, current?.bump?.bump, current?.bump?.companion]) {
      texture?.dispose();
    }
  }
}

/**
 * The proportions of the base file, or a square when nothing measured it.
 *
 * @param description - The texture as the backend resolved it.
 * @returns Width over height.
 */
function toTextureAspect(description: TextureDescription): number {
  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;

  return shape && shape.height > 0 ? shape.width / shape.height : 1;
}

/**
 * The two files the engine binds for a texture, when it binds a pair with both halves located.
 *
 * @param description - The texture as the backend resolved it.
 * @returns The bump and its companion, or null when there is no pair to draw.
 */
function selectBumpAssets(description: TextureDescription): Nullable<[XrayAsset, XrayAsset]> {
  const { bump } = description.material;

  if (!bump) {
    return null;
  }

  const located: Nullable<XrayAsset> = getLocatedAsset(bump.bump.resolution);
  const companion: Nullable<XrayAsset> = getLocatedAsset(bump.companion.resolution);

  return located && companion ? [located, companion] : null;
}
