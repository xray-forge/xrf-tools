import { Nullable } from "@xrf/types";

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
  /** The water, over the frame before the blended surfaces, and what it distorts moved once they are down. */
  readonly water: Nullable<IRendererPass>;
  readonly distortion: Nullable<IRendererPass>;
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
