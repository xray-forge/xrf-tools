import { IUniform, MeshStandardMaterial, Texture, WebGLProgramParametersWithUniforms } from "three";

import { Nullable } from "@/lib/types/general";

/**
 * Metres past which R1's detail modulation has faded to neutral, `r__dtex_range`
 * (`Layers/xrRender/TextureDescrManager.cpp`).
 */
export const XRAY_R1_DETAIL_RANGE: number = 50;

/**
 * The detail texture a surface modulates its diffuse with, uploaded, and how densely it lies over it.
 */
export interface IXrayDetail {
  texture: Texture;
  /** Times it repeats across the surface's base coordinate, from the base texture's descriptor. */
  scale: number;
}

/** Points a patched material at the detail texture to modulate with, or at none. */
export interface IXrayDetailShading {
  /**
   * Applies a detail texture and its tiling together, or takes the modulation off.
   *
   * @param detail - The uploaded texture and its tiling, or null to draw the surface undetailed.
   */
  setDetail(detail: Nullable<IXrayDetail>): void;
}

const FRAGMENT_PARS: string = `
uniform sampler2D xrayDetail;
uniform float xrayDetailScale;
uniform float xrayDetailEnabled;
`;

/**
 * The modulation the deferred `_d` shaders apply, in the shader's own spelling.
 *
 * `S.base.rgb = S.base.rgb * detail.rgb * 2` (`shaders/r2/sload.h`), over `tcdbump`, which the vertex shader builds as
 * `tcdh * dt_params` - the base coordinate times the descriptor's tiling, with no distance term in it.
 */
export const XRAY_DETAIL_FACTOR_GLSL: string = `
vec3 xrayDetailFactor = texture2D( xrayDetail, vMapUv * xrayDetailScale ).rgb * 2.0;
`;

/**
 * Modulates the diffuse the base texture just produced, which is where the engine's own `_d` shaders modulate it.
 */
const FRAGMENT_MAP: string = `
#include <map_fragment>
#ifdef USE_MAP
if ( xrayDetailEnabled > 0.5 ) {
${XRAY_DETAIL_FACTOR_GLSL}
  diffuseColor.rgb *= xrayDetailFactor;
}
#endif
`;

/**
 * What one texel of a detail texture multiplies the diffuse by, as the deferred shader computes it.
 *
 * @param texel - One channel of the detail texel, in `[0, 1]`.
 * @returns The factor the channel is multiplied by, one for the mid grey an average detail texture averages to.
 */
export function toXrayDetailFactor(texel: number): number {
  return texel * 2;
}

/**
 * Compiles X-Ray's detail modulation into a standard material.
 *
 * @param material - Material of a surface whose blender details it.
 * @returns The handle its detail texture is applied through.
 */
export function applyXrayDetailShading(material: MeshStandardMaterial): IXrayDetailShading {
  const texture: IUniform<Nullable<Texture>> = { value: null };
  const scale: IUniform<number> = { value: 1 };
  const enabled: IUniform<number> = { value: 0 };

  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms): void => {
    shader.uniforms.xrayDetail = texture;
    shader.uniforms.xrayDetailScale = scale;
    shader.uniforms.xrayDetailEnabled = enabled;

    shader.fragmentShader = shader.fragmentShader
      .replace("#include <common>", `#include <common>\n${FRAGMENT_PARS}`)
      .replace("#include <map_fragment>", FRAGMENT_MAP);
  };

  material.customProgramCacheKey = (): string => "xray-detail";
  material.needsUpdate = true;

  return {
    setDetail(detail: Nullable<IXrayDetail>): void {
      texture.value = detail?.texture ?? null;
      enabled.value = detail ? 1 : 0;

      if (detail) {
        scale.value = detail.scale;
      }
    },
  };
}
