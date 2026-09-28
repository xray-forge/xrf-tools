import { TMaterialTexel } from "./material-texel";

/** What the engine reconstructs from one texel of a bump pair, `sload.h` in TypeScript. */
export interface IMaterialBumpTexel {
  normal: [number, number, number];
  gloss: number;
  height: number;
}

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
export function decodeXrayBumpTexel(nu: TMaterialTexel, nuE: TMaterialTexel): IMaterialBumpTexel {
  return {
    normal: [nu[3] + (nuE[0] - 1), nu[2] + (nuE[1] - 1), nu[1] + (nuE[2] - 1)],
    gloss: nu[0] * nu[0],
    height: nuE[3],
  };
}
