import { IUniform, MeshStandardMaterial, Texture, WebGLProgramParametersWithUniforms } from "three";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { Nullable } from "@/lib/types/general";

/**
 * The two textures a bump declaration binds, uploaded.
 *
 * Always the pair: the engine's `sload_i` samples both every texel, and a `dummy` outcome has the real dummy files
 * uploaded here so the surface shows what the game shows.
 */
export interface IVisualBumpTextures {
  /** `normal.gloss`, the file the declaration names. */
  bump: Texture;
  /** `normal_error.height`, the `#` companion. */
  companion: Texture;
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
 * The bump decode of `gl/sload.h`, in the shader's own spelling.
 *
 * Held as one string so the GLSL patch and the TypeScript mirror below cannot drift: the test decodes a texel through
 * {@link decodeXrayBumpTexel} and pins that the compiled shader carries this exact expression.
 */
export const XRAY_BUMP_NORMAL_GLSL: string = "xrayNu.wzy + (xrayNuE.xyz - 1.0)";

/** Gloss is the bump's red channel squared, `S.gloss = Nu.x * Nu.x`. */
export const XRAY_BUMP_GLOSS_GLSL: string = "xrayNu.x * xrayNu.x";

/**
 * Reconstructs what the engine reads from one texel of a bump pair.
 *
 * `Nu.wzy` is (alpha, blue, green) of the bump, and the companion's rgb is the quantisation error the packer left, so
 * the normal is `Nu.wzy + (NuE.xyz - 1.0)` component by component. Not normalised, as the engine does not normalise
 * here either; the shader normalises after rotating through the tangent basis.
 *
 * @param nu - Texel of the bump, `normal.gloss`.
 * @param nuE - Texel of the companion, `normal_error.height`.
 * @returns Tangent-space normal, gloss, and height as the engine would hold them.
 */
export function decodeXrayBumpTexel(nu: TVisualTexel, nuE: TVisualTexel): IVisualBumpTexel {
  return {
    normal: [nu[3] + (nuE[0] - 1), nu[2] + (nuE[1] - 1), nu[1] + (nuE[2] - 1)],
    gloss: nu[0] * nu[0],
    height: nuE[2],
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

/** Switches a patched material between the flat and the bumped surface without recompiling it. */
export interface IVisualBumpShading {
  setEnabled(isEnabled: boolean): void;
}

/**
 * The vertex side of the patch: carries the authored tangent basis and the uv to the fragment stage.
 *
 * The basis is skinned with the normal when the mesh is, through the `skinMatrix` three.js has just built, and rotated
 * into view space by the same `normalMatrix`, so it stays the basis of the surface being drawn.
 */
const VERTEX_PARS: string = `
attribute vec3 xrayTangent;
attribute vec3 xrayBinormal;
varying vec3 vXrayTangent;
varying vec3 vXrayBinormal;
varying vec2 vXrayUv;
`;

const VERTEX_BEGIN: string = `
vec3 xrayObjectTangent = xrayTangent;
vec3 xrayObjectBinormal = xrayBinormal;
`;

const VERTEX_SKIN: string = `
#ifdef USE_SKINNING
xrayObjectTangent = vec4( skinMatrix * vec4( xrayObjectTangent, 0.0 ) ).xyz;
xrayObjectBinormal = vec4( skinMatrix * vec4( xrayObjectBinormal, 0.0 ) ).xyz;
#endif
`;

const VERTEX_TRANSFORM: string = `
vXrayTangent = normalMatrix * xrayObjectTangent;
vXrayBinormal = normalMatrix * xrayObjectBinormal;
vXrayUv = uv;
`;

const FRAGMENT_PARS: string = `
varying vec3 vXrayTangent;
varying vec3 vXrayBinormal;
varying vec2 vXrayUv;
uniform sampler2D xrayBump;
uniform sampler2D xrayBumpX;
uniform float xrayBumpEnabled;
`;

/**
 * Samples the pair once, before three.js reads roughness, so both the roughness and the normal below see one texel.
 *
 * Gloss stands in for roughness inverted: the engine feeds it to a specular power, and the closest the standard
 * material offers is a smoother surface where the bump says glossy.
 */
const FRAGMENT_ROUGHNESS: string = `
vec4 xrayNu = texture2D( xrayBump, vXrayUv );
vec4 xrayNuE = texture2D( xrayBumpX, vXrayUv );
float xrayGloss = ${XRAY_BUMP_GLOSS_GLSL};
#include <roughnessmap_fragment>
if ( xrayBumpEnabled > 0.5 ) {
  roughnessFactor = 1.0 - xrayGloss;
}
`;

/**
 * The decoded normal rotated through the authored basis, the way `deffer_model_bump.vs` feeds `M1..M3` to the pixel
 * shader and `deffer_base_bump.ps` multiplies: tangent by x, binormal by y, normal by z.
 */
const FRAGMENT_NORMAL: string = `
#include <normal_fragment_maps>
if ( xrayBumpEnabled > 0.5 ) {
  vec3 xrayNormalTangentSpace = ${XRAY_BUMP_NORMAL_GLSL};
  normal = normalize(
    normalize( vXrayTangent ) * xrayNormalTangentSpace.x
    + normalize( vXrayBinormal ) * xrayNormalTangentSpace.y
    + normal * xrayNormalTangentSpace.z
  );
}
`;

/**
 * Shades a standard material the way the engine shades a bumped X-Ray surface.
 *
 * An `onBeforeCompile` patch rather than a `ShaderMaterial`, so the viewer's lights, tone mapping, wireframe and
 * checkerboard keep working and the toggle compares like with like: exactly two things change, the normal and the
 * roughness, both read from the pair through `sload.h`'s decode. Never assigns the bump to `normalMap`, whose packing
 * is not X-Ray's.
 *
 * The switch is a uniform, so toggling costs no recompile and no re-upload; the patched program is cached under its
 * own key so it never shares one with an unpatched standard material.
 *
 * @param material - Material of a mesh whose geometry carries `xrayTangent` and `xrayBinormal` attributes.
 * @param textures - The uploaded pair to sample.
 * @returns The switch between the flat and the bumped surface.
 */
export function applyXrayBumpShading(
  material: MeshStandardMaterial,
  textures: IVisualBumpTextures
): IVisualBumpShading {
  const enabled: IUniform<number> = { value: 1 };

  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms): void => {
    shader.uniforms.xrayBump = { value: textures.bump };
    shader.uniforms.xrayBumpX = { value: textures.companion };
    shader.uniforms.xrayBumpEnabled = enabled;

    shader.vertexShader = shader.vertexShader
      .replace("#include <common>", `#include <common>\n${VERTEX_PARS}`)
      .replace("#include <beginnormal_vertex>", `#include <beginnormal_vertex>\n${VERTEX_BEGIN}`)
      .replace("#include <skinnormal_vertex>", `#include <skinnormal_vertex>\n${VERTEX_SKIN}`)
      .replace("#include <defaultnormal_vertex>", `#include <defaultnormal_vertex>\n${VERTEX_TRANSFORM}`);

    shader.fragmentShader = shader.fragmentShader
      .replace("#include <common>", `#include <common>\n${FRAGMENT_PARS}`)
      .replace("#include <roughnessmap_fragment>", FRAGMENT_ROUGHNESS)
      .replace("#include <normal_fragment_maps>", FRAGMENT_NORMAL);
  };

  material.customProgramCacheKey = (): string => "xray-bump";
  material.needsUpdate = true;

  return {
    setEnabled(isEnabled: boolean): void {
      enabled.value = isEnabled ? 1 : 0;
    },
  };
}
