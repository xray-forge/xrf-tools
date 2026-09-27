import { Nullable } from "@xrf/types";

import { IBaseFramePasses } from "#/graph/base-frame-passes";
import { IOcclusionFramePasses } from "#/graph/occlusion-frame-passes";
import { IRendererPass } from "#/pass/renderer-pass";

/** The optional passes a frame holds now, each null or empty while its feature is off. */
export interface IFrameOptionalPasses {
  readonly grass: Nullable<IRendererPass>;
  /** The frame combine finished measured, and the exposure adapted towards it. */
  readonly exposure: Nullable<IRendererPass>;
  /** The static draws the first phase's depth hid culled again and drawn, and the depth reduced after. */
  readonly occlusion: Nullable<IOcclusionFramePasses>;
  readonly motionBackground: Nullable<IRendererPass>;
  readonly shadows: ReadonlyArray<IRendererPass>;
  readonly lightShadows: Nullable<IRendererPass>;
  readonly lights: Nullable<IRendererPass>;
  readonly ambientOcclusion: Nullable<IRendererPass>;
  /** A temporal resolve, and the passes it needs drawn before the blended surfaces. */
  readonly resolve: Nullable<IRendererPass & { readonly beforeBlended: ReadonlyArray<IRendererPass> }>;
  readonly smoothing: Nullable<IRendererPass>;
  readonly spatial: Nullable<IRendererPass>;
  readonly sharpen: Nullable<IRendererPass>;
}

/**
 * The frame's passes in order: the grass into the G-buffer, what its depth hid culled again and drawn into it, the
 * sky's motion once the G-buffer is whole, its depth reduced for the next frame, the shadow
 * cascades and light faces before the sun reads them, the local lights after it, the occlusion before combine, what the
 * resolve needs before the water and the blended surfaces, what the water distorts moved once they are down, and the
 * upscaling and its sharpening before the helpers. The smoothing of
 * a mode that does not jitter comes after the helpers, which it smooths too, unless FSR 1 upscales what it smoothed.
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
    base.gbuffer,
    ...some(optional.grass),
    ...(occlusion ? [occlusion.lateCull, occlusion.gbufferLate] : []),
    ...some(optional.motionBackground),
    ...some(occlusion?.pyramid ?? null),
    base.wallmarks,
    ...optional.shadows,
    ...some(optional.lightShadows),
    base.sun,
    ...some(optional.lights, optional.ambientOcclusion),
    base.combine,
    ...some(optional.exposure),
    ...(resolve?.beforeBlended ?? []),
    base.water,
    base.forward,
    base.distortion,
    ...some(resolve),
    ...(spatial ? some(smoothing, spatial) : []),
    ...some(optional.sharpen),
    base.overlay,
    ...(spatial ? [] : some(smoothing)),
    present,
  ];
}
