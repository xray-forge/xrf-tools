import { MeshStandardMaterial, WebGLProgramParametersWithUniforms } from "three";

import { applyRenderPatch } from "@/core/render/lib/surface/render-patch";

/**
 * Writes the surface's own colour, with nothing the scene's lights did to it.
 */
const FRAGMENT_UNLIT: string = `
#include <opaque_fragment>
gl_FragColor = vec4( diffuseColor.rgb, diffuseColor.a );
`;

/**
 * Draws a surface without the scene's lighting, the way the engine's forward passes draw one.
 *
 * @param material - Material of a surface whose pass the engine draws unlit.
 */
export function applyXrayUnlitShading(material: MeshStandardMaterial): void {
  applyRenderPatch(material, { name: "xray-unlit" }, (shader: WebGLProgramParametersWithUniforms): void => {
    shader.fragmentShader = shader.fragmentShader.replace("#include <opaque_fragment>", FRAGMENT_UNLIT);
  });
}
