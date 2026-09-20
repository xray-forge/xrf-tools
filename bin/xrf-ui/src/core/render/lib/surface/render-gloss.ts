import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { applyRenderPatch } from "@/core/render/lib/surface/render-patch";

/**
 * What every deferred base shader writes into the gloss channel for a surface that declares none: `def_gloss`, two of
 * two hundred and fifty five (`shaders/r2/common.h`).
 */
export const XRAY_DEFAULT_GLOSS: number = 2 / 255;

/**
 * The fragment local the specular is scaled by, which a patch reading gloss out of a texture writes its own value
 * into.
 */
export const XRAY_GLOSS_VARIABLE: string = "xrayGloss";

/** Guards the declaration, so every patch needing the local can carry one and only the first of them is compiled. */
const XRAY_GLOSS_GUARD: string = "XRAY_GLOSS_DECLARED";

/** Decimals the constant is compiled in with, enough for a channel that only ever carries eight bits. */
const GLOSS_PRECISION: number = 6;

/**
 * The declaration of the gloss local, guarded so more than one patch may carry it.
 *
 * @param gloss - The gloss to fall back to where nothing writes one.
 * @returns The declaration, ready to splice in.
 */
export function toXrayGlossDeclaration(gloss: number = XRAY_DEFAULT_GLOSS): string {
  return `
#ifndef ${XRAY_GLOSS_GUARD}
#define ${XRAY_GLOSS_GUARD}
float ${XRAY_GLOSS_VARIABLE} = ${gloss.toFixed(GLOSS_PRECISION)};
#endif
`;
}

/**
 * Scales the whole specular lobe by the engine's gloss, which is what the deferred light pass does with the channel.
 */
const FRAGMENT_SPECULAR: string = `
#include <lights_physical_fragment>
material.specularColor *= ${XRAY_GLOSS_VARIABLE};
material.specularColorBlended *= ${XRAY_GLOSS_VARIABLE};
material.specularF90 *= ${XRAY_GLOSS_VARIABLE};
`;

/**
 * Compiles the engine's gloss into a standard material, as a constant rather than as a uniform.
 *
 * @param material - Material to shade.
 * @param gloss - The gloss the surface's own shader writes, `def_gloss` for everything that writes no other.
 */
export function applyXrayGlossShading(material: MeshStandardMaterial, gloss: number = XRAY_DEFAULT_GLOSS): void {
  const identity = { key: `xray-gloss:${gloss.toFixed(GLOSS_PRECISION)}`, name: "xray-gloss" };

  applyRenderPatch(material, identity, (shader: WebGLProgramParametersWithUniforms): void => {
    shader.fragmentShader = shader.fragmentShader
      // The first statement of `main`, so everything a later chunk replaces can read and write it.
      .replace(
        "#include <clipping_planes_fragment>",
        `#include <clipping_planes_fragment>${toXrayGlossDeclaration(gloss)}`
      )
      .replace("#include <lights_physical_fragment>", FRAGMENT_SPECULAR);
  });
}
