import { Injectable, OnDeactivation } from "@wirestate/core";
import { Observable, RefObservable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { formatDuration } from "@xrf/format";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { fetchBulk } from "@/core/ipc/bulk";
import { texturesBulkRoutes } from "@/core/ipc/commands/textures-bulk";
import { EAssetSource, TextureDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureBumpAssets,
  ITextureSurfaceFiles,
  selectTextureBumpAssets,
  toTextureAspect,
} from "@/core/textures/lib/texture-surface";
import { ITextureTexels, readTextureTexels } from "@/core/textures/lib/texture-texels";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * Reads what the panels show of the selected texture beside its body: the bump pair's texels, which the channels panel
 * draws its planes from and reads a texel off. The body itself is drawn by the native renderer, which reads its files.
 */
@Injectable()
export class TextureSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @RefObservable()
  public files: AsyncState<ITextureSurfaceFiles> = AsyncState.idle(EMPTY_TEXTURE_SURFACE);

  /** Which texture the files were read for, so a view can tell an answer for the one on screen from a stale one. */
  @Observable()
  public reference: Nullable<string> = null;

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Reads the pair one texture's material binds.
   *
   * @param description - The texture as the backend resolved it, whose roots address the reads.
   */
  @LatestFlow("files")
  public *load(description: TextureDescription): TFlow {
    const timer: Timer = new Timer();

    this.files = this.files.asLoading(EMPTY_TEXTURE_SURFACE);
    this.reference = null;

    try {
      const { roots } = description;
      const assets: Nullable<ITextureBumpAssets> = selectTextureBumpAssets(description);
      const bump: Nullable<ITextureTexels> = assets
        ? yield* call(this.readTexels(roots, assets.bump.logicalPath))
        : null;
      const companion: Nullable<ITextureTexels> =
        bump && assets ? yield* call(this.readTexels(roots, assets.companion.logicalPath)) : null;

      this.files = this.files.asReady({
        aspect: toTextureAspect(description),
        // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
        bump: bump && companion ? { bump, companion } : null,
      });
      this.reference = description.reference;

      this.log.info(
        "Texture surface read:",
        description.reference,
        { bump: Boolean(bump), companion: Boolean(companion) },
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

  public clear(): void {
    cancelFlow(this, "files");

    runInAction(() => {
      this.files = this.files.asIdle(EMPTY_TEXTURE_SURFACE);
      this.reference = null;
    });
  }

  /**
   * Reads one half of the pair as texels, decoded by the backend from whatever layout it is stored in.
   *
   * @param roots - Roots the description was resolved in, so the read reaches the same file.
   * @param logicalPath - Engine identity of the file, an asset of those roots.
   * @returns Its texels, or nothing for a read that failed.
   */
  private async readTexels(roots: XrayRoots, logicalPath: string): Promise<Nullable<ITextureTexels>> {
    try {
      return readTextureTexels(
        await fetchBulk(texturesBulkRoutes.readTexels(roots, { kind: EAssetSource.ASSET, logicalPath }))
      );
    } catch (error: unknown) {
      this.log.error(`Failed to read '${logicalPath}':`, transformError(error));

      return null;
    }
  }
}
