import { Nullable } from "@xrf/types";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XrayMaterialDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { EVisualTextureState, IVisualTextureFile } from "@/core/visuals/lib/visual-texture";

/** The two files of a bump pair, which are only ever read, uploaded and drawn together. */
export interface IVisualBumpFiles {
  bump: IVisualTextureFile;
  companion: IVisualTextureFile;
}

/** A submesh whose material binds a bump pair, and the two located files to fetch for it. */
export interface ILoadableBump {
  submeshIndex: number;
  bump: string;
  companion: string;
}

/**
 * What became of one submesh's bump inputs on the frontend, each half on its own.
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

/** What the engine reconstructs from one texel of the pair, `sload.h:144` in TypeScript. */
export interface IVisualBumpTexel {
  normal: [number, number, number];
  gloss: number;
  height: number;
}

/** One rgba sample, each channel in `[0, 1]` as a shader reads it. */
export type TVisualTexel = readonly [number, number, number, number];

/**
 * Reconstructs what the engine reads from one texel of a bump pair.
 *
 * `Nu.wzy` is (alpha, blue, green) of the bump, and the companion's rgb is the quantisation error the packer left, so
 * the normal is `Nu.wzy + (NuE.xyz - 1.0)` component by component. Not normalised, as the engine does not normalise
 * here either; the shader normalises after rotating through the tangent basis.
 *
 * Height comes from the companion's alpha, which is what the generator writes and what parallax samples, rather than
 * `NuE.z`, which `surface_bumped` assigns and which is the normal's z error.
 *
 * @param nu - Texel of the bump, `normal.gloss`.
 * @param nuE - Texel of the companion, `normal_error.height`.
 * @returns Tangent-space normal, gloss, and height as the engine would hold them.
 */
export function decodeXrayBumpTexel(nu: TVisualTexel, nuE: TVisualTexel): IVisualBumpTexel {
  return {
    normal: [nu[3] + (nuE[0] - 1), nu[2] + (nuE[1] - 1), nu[1] + (nuE[2] - 1)],
    gloss: nu[0] * nu[0],
    height: nuE[3],
  };
}

/**
 * Submeshes whose material binds a bump pair with both files located, addressed by those files.
 *
 * Joined by the declared reference, which is how the backend keyed the materials. Only a pair with both halves located
 * is loadable: the engine binds both or, when a name has no dummy, draws its placeholder, and the placeholder pair is
 * what the panel reports rather than something worth uploading.
 *
 * @param textures - The model's texture references, resolved or not.
 * @param materials - What the renderer builds for each reference.
 * @returns Every submesh with a complete pair to fetch.
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
