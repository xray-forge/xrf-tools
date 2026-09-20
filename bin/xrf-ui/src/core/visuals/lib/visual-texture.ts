import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { EXrayResolution, XrayAsset, XrayResolution } from "@/core/ipc/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

/**
 * Why a submesh ended up without a texture on screen, or that it has one.
 */
export enum EVisualTextureState {
  /** The submesh declares no texture, which is normal for a skeleton's own record. */
  ABSENT = "absent",
  /** Bytes are still on the way. */
  LOADING = "loading",
  /** Uploaded and applied, in the layout the file stores. */
  APPLIED = "applied",
  /**
   * Applied, but expanded by the backend first because the renderer cannot upload this layout.
   *
   * Distinct from `APPLIED` because the upload is not the file: it arrives as one png, so it carries no mip chain
   * whatever the file's header says, and costs the memory of raw pixels rather than of blocks.
   */
  DECODED = "decoded",
  /** Located, but stored in a format neither the renderer nor the backend can read. */
  UNSUPPORTED_FORMAT = "unsupportedFormat",
  /** Nothing to load: no source was searchable, or neither the reference nor the engine's dummy resolved. */
  UNRESOLVED = "unresolved",
  /** Located, but reading or parsing the file failed, or the reference was not a usable one. */
  FAILED = "failed",
}

/**
 *  What became of one submesh's texture on the frontend, paired with what the backend resolved.
 */
export interface IVisualTextureStatus {
  submeshIndex: number;
  state: EVisualTextureState;
  /** Present when the state is `FAILED`, so a panel can say why rather than only that. */
  reason: Nullable<string>;
}

/** A submesh texture whose bytes can be fetched, and the located file to fetch them from. */
export interface ILoadableTexture {
  submeshIndex: number;
  logicalPath: string;
}

/**
 * Submeshes worth fetching bytes for, paired with the logical path to fetch.
 *
 * The path comes from the outcome rather than from the reference, so the read lands on the file resolution named — a
 * substituted dummy included — instead of resolving a second time and possibly differently.
 */
export function toLoadableTextures(textures: Array<VisualTextureDependency>): Array<ILoadableTexture> {
  return textures.flatMap((texture) => {
    const asset: Nullable<XrayAsset> = getLocatedAsset(texture.resolution);

    return asset ? [{ submeshIndex: texture.submeshIndex, logicalPath: asset.logicalPath }] : [];
  });
}

/**
 * The state a submesh starts in, before any bytes are asked for.
 *
 * A rejected reference is a failure rather than an absence: the name in the mesh header is unusable, which is worth
 * saying rather than showing the submesh as having nothing to load.
 */
export function toInitialTextureState(resolution: XrayResolution): EVisualTextureState {
  if (getLocatedAsset(resolution)) {
    return EVisualTextureState.LOADING;
  }

  return resolution.kind === EXrayResolution.REJECTED ? EVisualTextureState.FAILED : EVisualTextureState.UNRESOLVED;
}
