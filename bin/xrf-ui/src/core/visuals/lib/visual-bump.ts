import { Nullable } from "@xrf/types";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XrayMaterialDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";

/** A submesh whose material binds a bump pair, and the two located files the renderer reads for it. */
export interface ILoadableBump {
  submeshIndex: number;
  bump: string;
  companion: string;
}

/**
 * What became of one submesh's bump inputs, each half on its own.
 *
 * Separate rather than one state because a companion that fails to decode does not cost the bump, and the panel says
 * which half is the problem.
 */
export interface IVisualBumpStatus {
  submeshIndex: number;
  bump: EVisualTextureState;
  companion: EVisualTextureState;
  /** Present when either half is `FAILED`, so a panel can say why rather than only that. */
  reason: Nullable<string>;
}

/**
 * Submeshes whose material binds a bump pair with both files located, addressed by those files.
 *
 * Joined by the declared reference, which is how the backend keyed the materials. Only a pair with both halves located
 * is loadable: the engine binds both or, when a name has no dummy, draws its placeholder, and the placeholder pair is
 * what the panel reports rather than something drawn.
 *
 * @param textures - The model's texture references, resolved or not.
 * @param materials - What the renderer builds for each reference.
 * @returns Every submesh with a complete pair.
 */
export function toLoadableBumps(
  textures: Array<VisualTextureDependency>,
  materials: Record<string, XrayMaterialDescriptor>
): Array<ILoadableBump> {
  return textures.flatMap((texture) => {
    const bump = materials[texture.reference]?.bump;

    if (!bump) {
      return [];
    }

    const located: Nullable<XrayAsset> = getLocatedAsset(bump.bump.resolution);
    const companion: Nullable<XrayAsset> = getLocatedAsset(bump.companion.resolution);

    return located && companion
      ? [{ submeshIndex: texture.submeshIndex, bump: located.logicalPath, companion: companion.logicalPath }]
      : [];
  });
}
