import { Nullable } from "@xrf/types";

import { IBaseFramePasses } from "#/graph/base-frame-passes";
import { IFrameOptionalPasses } from "#/graph/frame-optional-passes";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * The frame's passes in order: the static draws into the G-buffer, what their depth hid culled again and drawn into
 * it, their depth reduced for the next frame, and only then the plain draws and the grass, which move with no version
 * the culls see; the sky's motion once the G-buffer is whole, the shadow cascades and light faces before the sun reads
 * them, the local lights after it, the occlusion before combine, what the resolve needs before the water and the
 * blended surfaces, what the water distorts moved once they are down, and the upscaling and its sharpening before the
 * helpers. The smoothing of a mode that does not jitter comes after the helpers, which it smooths too, unless FSR 1
 * upscales what it smoothed.
 *
 * @param base - The passes every frame draws.
 * @param optional - The ones the features add.
 * @param present - The pass putting the frame on the canvas.
 * @returns Every pass, in the order they draw.
 */
export function toFramePassOrder(
  base: IBaseFramePasses,
  optional: IFrameOptionalPasses,
  present: IRendererPass
): Array<IRendererPass> {
  const { occlusion, resolve, smoothing, spatial } = optional;

  function some(...passes: ReadonlyArray<Nullable<IRendererPass>>): Array<IRendererPass> {
    return passes.filter((pass: Nullable<IRendererPass>): pass is IRendererPass => pass !== null);
  }

  return [
    base.cull,
    base.gbufferEarly,
    ...(occlusion ? [occlusion.lateCull, occlusion.gbufferLate, occlusion.pyramid] : []),
    base.gbuffer,
    ...some(optional.grass, optional.motionBackground),
    base.wallmarks,
    ...optional.shadows,
    ...some(optional.lightShadows),
    base.rainCover,
    base.sun,
    ...some(optional.lights, optional.ambientOcclusion),
    base.combine,
    ...some(optional.exposure),
    ...(resolve?.beforeBlended ?? []),
    ...some(optional.water),
    base.forward,
    base.rain,
    // It moves what the water wrote, so it draws only where the water does.
    ...(optional.water ? some(optional.distortion) : []),
    ...some(resolve),
    ...(spatial ? some(smoothing, spatial) : []),
    ...some(optional.sharpen),
    base.overlay,
    ...(spatial ? [] : some(smoothing)),
    present,
  ];
}
