import { Discard, Fn, If, screenUV, texture, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toBaseLitColor } from "#/shader/base-lighting.tsl";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * @param gbuffer - The G-buffer combined.
 * @param light - What the lights accumulated.
 * @param uniforms - What the frame's shaders read.
 * @returns Every drawn pixel lit, fogged and tonemapped; where nothing was drawn, nothing.
 */
export function toCombinePassFragment(
  gbuffer: IGBufferTextures,
  light: Texture,
  uniforms: RendererUniforms
): Node<"vec4"> {
  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(gbuffer, uniforms.camera);

    // Where nothing was drawn the backdrop the target was cleared to shows.
    If(sample.depth.greaterThanEqual(1), () => {
      Discard();
    });

    return vec4(
      toBaseLitColor(sample.albedo, sample.gloss, sample.hemi, texture(light, screenUV), sample.point, uniforms),
      1
    );
  })();
}
