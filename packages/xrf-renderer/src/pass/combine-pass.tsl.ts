import { Nullable } from "@xrf/types";
import {
  Discard,
  float,
  Fn,
  getViewPosition,
  If,
  mix,
  normalize,
  screenCoordinate,
  screenUV,
  select,
  texture,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toUpsampledAmbientOcclusion } from "#/shader/ambient-occlusion.tsl";
import { toBaseShadedColor, toFinishedColor, toFogAmount, toFogColor } from "#/shader/base-lighting.tsl";
import { toSkyWithClouds } from "#/shader/clouds.tsl";
import { toOutputDither } from "#/shader/dither.tsl";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { ISkyWithCloudsUniforms } from "#/shader/sky-with-clouds-uniforms";
import { toSkyColor, toSkyHaze } from "#/shader/sky.tsl";
import { toToneMapped, toUntoneMapped } from "#/shader/tonemap.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * @param gbuffer - The G-buffer combined.
 * @param light - What the lights accumulated.
 * @param uniforms - What the frame's shaders read.
 * @param ambientOcclusion - The screen's occlusion at half resolution, or none.
 * @returns Every drawn pixel lit, fogged and tonemapped, and faded into the sky as `combine_1` blends it over the sky
 *   by the fog squared; where nothing was drawn, the sky, total fog or nothing.
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
    // A level's frame draws its sky behind everything, so an empty pixel is the sky there instead.
    const isSkyDrawn: Node<"bool"> = uniforms.sky.drawn.mul(uniforms.settings.lit).greaterThan(0.5);

    If(isEmpty.and(isFogged.not()).and(isSkyDrawn.not()), () => {
      Discard();
    });

    const shaded: Node<"vec3"> = toBaseShadedColor(
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
    const isLit: Node<"bool"> = uniforms.settings.lit.greaterThan(0.5);
    const lit: Node<"vec3"> = select(isLit, toFinishedColor(shaded, sample.point.position, uniforms), sample.albedo);

    const toPixel: Node<"vec3"> = getViewPosition(screenUV, float(0.5), uniforms.camera.projectionInverse);
    const direction: Node<"vec3"> = normalize(uniforms.camera.viewToWorld.mul(vec4(toPixel, 0)).xyz);
    const skies: ISkyWithCloudsUniforms = {
      clouds: uniforms.clouds,
      engine: uniforms.engine,
      scale: uniforms.exposure.scale,
      sky: uniforms.sky,
    };
    const sky: Node<"vec3"> = toSkyWithClouds(direction, toSkyColor(direction, skies), skies);
    const fog: Node<"float"> = toFogAmount(sample.point.position, uniforms);
    // The engine fogs towards `fog_color`, then fades into the sky itself by the fog squared.
    const faded: Node<"vec3"> = select(isSkyDrawn, mix(lit, sky, fog.mul(fog)), lit).toVar();

    // The haze takes the place of both, by the engine's own two blends: the distance takes the colour of the sky behind
    // it, no cloud's shape showing, the fog towards the haze's lit colour before the tonemap.
    If(uniforms.sky.hazed.greaterThan(0.5).and(isSkyDrawn).and(isEmpty.not()), () => {
      const { scale } = uniforms.exposure;
      const haze: Node<"vec3"> = toSkyHaze(direction, uniforms.sky);
      const fogged: Node<"vec3"> = select(
        isLit,
        toToneMapped(mix(shaded, toUntoneMapped(haze, scale), fog), scale),
        mix(sample.albedo, haze, fog)
      );

      faded.assign(mix(fogged, haze, fog.mul(fog)));
    });

    const shown: Node<"vec3"> = select(isEmpty, select(isSkyDrawn, sky, toFogColor(uniforms)), faded);

    // The frame is eight bits a channel from here, and the sky's haze and the fog are gradients a step wide.
    return vec4(shown.add(toOutputDither(screenCoordinate.xy)), 1);
  })();
}
