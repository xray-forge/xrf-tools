import { IUniform, MeshStandardMaterial, Texture, Vector2, WebGLProgramParametersWithUniforms } from "three";

import { Nullable } from "@/lib/types/general";

/**
 * Metres past which a detail texture has faded to neutral, `r__dtex_range` (`Layers/xrRender/TextureDescrManager.cpp`).
 */
export const XRAY_DETAIL_RANGE: number = 50;

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
uniform vec2 xrayDetailParams;
uniform float xrayDetailEnabled;
`;

/**
 * `calc_detail` and the modulation the `_dt` shaders apply with it, in the shader's own spelling.
 */
export const XRAY_DETAIL_FACTOR_GLSL: string = `
float xrayDetailRange = length( vViewPosition ) * xrayDetailParams.y;
float xrayDetailFade = min( xrayDetailRange * xrayDetailRange, 1.0 );
vec3 xrayDetailTexel = texture2D( xrayDetail, vMapUv * xrayDetailParams.x ).rgb;
vec3 xrayDetailFactor = ( xrayDetailTexel * ( 1.0 - xrayDetailFade ) + 0.5 * xrayDetailFade ) * 2.0;
`;

/**
 * Modulates the diffuse the base texture just produced, which is where the engine's own `_dt` shaders modulate it.
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
 * What one texel of a detail texture multiplies the diffuse by at a distance, as the shader computes it.
 *
 * @param texel - One channel of the detail texel, in `[0, 1]`.
 * @param distance - Distance from the camera to the surface, in metres.
 * @returns The factor the channel is multiplied by, which is one wherever the detail has faded out.
 */
export function toXrayDetailFactor(texel: number, distance: number): number {
  const range: number = distance / XRAY_DETAIL_RANGE;
  const fade: number = Math.min(range * range, 1);

  return (texel * (1 - fade) + 0.5 * fade) * 2;
}

/**
 * Compiles X-Ray's detail modulation into a standard material.
 *
 * @param material - Material of a surface whose blender details it.
 * @returns The handle its detail texture is applied through.
 */
export function applyXrayDetailShading(material: MeshStandardMaterial): IXrayDetailShading {
  const texture: IUniform<Nullable<Texture>> = { value: null };
  const params: IUniform<Vector2> = { value: new Vector2(1, 1 / XRAY_DETAIL_RANGE) };
  const enabled: IUniform<number> = { value: 0 };

  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms): void => {
    shader.uniforms.xrayDetail = texture;
    shader.uniforms.xrayDetailParams = params;
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
        params.value.setX(detail.scale);
      }
    },
  };
}
