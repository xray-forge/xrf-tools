import { Fn, normalize, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toSunLight } from "#/shader/base-lighting.tsl";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { toSunShadow } from "#/shader/sun-shadow.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * @param gbuffer - The G-buffer lit.
 * @param uniforms - What the frame's shaders read.
 * @param shadows - Each cascade's shadow map.
 * @returns The sun at every pixel, times how much of it reaches the pixel: diffuse in colour, specular in alpha.
 */
export function toSunPassFragment(
  gbuffer: IGBufferTextures,
  uniforms: RendererUniforms,
  shadows: ReadonlyArray<Texture>
): Node<"vec4"> {
  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(gbuffer, uniforms.camera);
    const { viewToWorld } = uniforms.camera;
    const position: Node<"vec3"> = viewToWorld.mul(vec4(sample.point.position, 1)).xyz;
    const normal: Node<"vec3"> = normalize(viewToWorld.mul(vec4(sample.point.normal, 0)).xyz);

    return toSunLight(sample.point, uniforms).mul(toSunShadow(position, normal, uniforms.shadows, shadows));
  })();
}
