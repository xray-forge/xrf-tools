import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

/**  What a compiled level's second texture actually carries, and why it is not a light map. */
export const XRAY_HEMI_CHANNEL: string = "a";

/** Three.js samples an ambient occlusion map's red channel; the engine's hemisphere term is in the alpha. */
const AO_FRAGMENT: string = `
#ifdef USE_AOMAP
  float ambientOcclusion = ( texture2D( aoMap, vAoMapUv ).${XRAY_HEMI_CHANNEL} - 1.0 ) * aoMapIntensity + 1.0;
  reflectedLight.indirectDiffuse *= ambientOcclusion;
  #if defined( USE_CLEARCOAT )
    clearcoatSpecularIndirect *= ambientOcclusion;
  #endif
  #if defined( USE_SHEEN )
    sheenSpecularIndirect *= ambientOcclusion;
  #endif
  #if defined( USE_ENVMAP ) && defined( STANDARD )
    float dotNV = saturate( dot( geometryNormal, geometryViewDir ) );
    reflectedLight.indirectSpecular *= computeSpecularOcclusion( dotNV, ambientOcclusion, material.roughness );
  #endif
#endif
`;

/**
 * Compiles the engine's reading of a level's second texture into a standard material.
 *
 * @param material - Material of a surface whose shader table names a second texture.
 */
export function applyXrayHemiShading(material: MeshStandardMaterial): void {
  const compile = material.onBeforeCompile;
  const key = material.customProgramCacheKey;

  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms, renderer: never): void => {
    compile?.call(material, shader, renderer);

    shader.fragmentShader = shader.fragmentShader.replace("#include <aomap_fragment>", AO_FRAGMENT);
  };

  // Named in the key, because three.js caches a program by its parameters and this one: two materials that agree on
  // every parameter but not on their patches would otherwise be handed each other's compiled shader.
  material.customProgramCacheKey = (): string => `${key.call(material)}|xray-hemi`;
}
