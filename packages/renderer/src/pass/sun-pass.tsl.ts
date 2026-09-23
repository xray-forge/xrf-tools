import { Fn } from "three/tsl";
import { Node } from "three/webgpu";

import { toSunLight } from "#/shader/base-lighting.tsl";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * @param gbuffer - The G-buffer lit.
 * @param uniforms - What the frame's shaders read.
 * @returns The sun at every pixel: diffuse in colour, specular in alpha.
 */
export function toSunPassFragment(gbuffer: IGBufferTextures, uniforms: RendererUniforms): Node<"vec4"> {
  // todo: multiply by the sun shadow once the cached cascades exist.
  return Fn(() => toSunLight(readGBuffer(gbuffer, uniforms.camera).point, uniforms))();
}
