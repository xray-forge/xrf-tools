import { Nullable } from "@xrf/types";
import { Discard, float, Fn, If, screenUV, select, texture, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toUpsampledAmbientOcclusion } from "#/shader/ambient-occlusion.tsl";
import { toBaseLitColor, toFogColor } from "#/shader/base-lighting.tsl";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * @param gbuffer - The G-buffer combined.
 * @param light - What the lights accumulated.
 * @param uniforms - What the frame's shaders read.
 * @param ambientOcclusion - The screen's occlusion at half resolution, or none.
 * @returns Every drawn pixel lit, fogged and tonemapped; where nothing was drawn, total fog or nothing.
 */
export function toCombinePassFragment(
  gbuffer: IGBufferTextures,
  light: Texture,
  uniforms: RendererUniforms,
  ambientOcclusion: Nullable<Texture>
): Node<"vec4"> {
  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(gbuffer, uniforms.camera);
    const isEmpty: Node<"bool"> = sample.depth.lessThanEqual(0);
    // The far plane ends where fog is total, so in a lit and fogged frame an empty pixel is what anything past it
    // would have come to. Otherwise the backdrop the target was cleared to shows.
    const isFogged: Node<"bool"> = uniforms.lighting.fogged.mul(uniforms.settings.lit).greaterThan(0.5);

    If(isEmpty.and(isFogged.not()), () => {
      Discard();
    });

    const lit: Node<"vec3"> = toBaseLitColor(
      sample.albedo,
      sample.gloss,
      sample.hemi,
      texture(light, screenUV),
      sample.point,
      uniforms,
      ambientOcclusion
        ? toUpsampledAmbientOcclusion(ambientOcclusion, sample.point.position.z.negate(), isEmpty.not())
        : float(1)
    );

    return vec4(select(isEmpty, toFogColor(uniforms), lit), 1);
  })();
}
