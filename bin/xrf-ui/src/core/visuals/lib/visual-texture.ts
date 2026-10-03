import { Nullable } from "@xrf/types";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { EXrayResolution, XrayResolution } from "@/core/ipc/types/xrf-vfs";

/**
 * Why a submesh ended up without a texture on screen, or that it has one.
 */
export enum EVisualTextureState {
  /** The submesh declares no texture, which is normal for a skeleton's own record. */
  ABSENT = "absent",
  /** The renderer has not read it yet. */
  LOADING = "loading",
  /** Drawn, uploaded in the layout the file stores. */
  APPLIED = "applied",
  /**
   * Drawn, but expanded to eight bits a channel first because the GPU cannot sample this layout as stored.
   *
   * Distinct from `APPLIED` because the upload is not the file: it costs the memory of raw pixels rather than of blocks.
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

/**
 * The state a submesh starts in, before the renderer reads anything.
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
