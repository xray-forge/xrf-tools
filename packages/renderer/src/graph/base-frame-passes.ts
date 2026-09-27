import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCullPass } from "#/pass/static-cull-pass";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { WaterDistortionPass } from "#/pass/water-distortion-pass";
import { WaterPass } from "#/pass/water-pass";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { StaticCull } from "#/scene/static/static-cull";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** `Base`'s passes every frame draws, by their place in it. */
export interface IBaseFramePasses {
  readonly cull: IRendererPass;
  readonly gbuffer: IRendererPass;
  readonly wallmarks: IRendererPass;
  readonly sun: IRendererPass;
  readonly combine: CombinePass;
  readonly water: WaterPass;
  readonly forward: IRendererPass;
  readonly distortion: WaterDistortionPass;
  readonly overlay: OverlayPass;
}

/**
 * `Base`'s frame, in R4's order: the static draws culled, the G-buffer, wall marks into its albedo, the sun, combine
 * with its fog and tonemap, then the water and the blended surfaces over the tonemapped frame, what the water distorts
 * moved, and the helpers over it all.
 *
 * @param targets - What the passes draw into.
 * @param uniforms - What their shaders read.
 * @param overlays - The helpers drawn last.
 * @param cull - What culls the static draws.
 * @returns The passes, by their place in the frame.
 */
export function createBaseFramePasses(
  targets: RendererTargets,
  uniforms: RendererUniforms,
  overlays: RendererOverlays,
  cull: StaticCull
): IBaseFramePasses {
  return {
    combine: new CombinePass(targets, uniforms),
    cull: new StaticCullPass(cull),
    distortion: new WaterDistortionPass(targets, uniforms),
    forward: new ForwardPass(targets),
    gbuffer: new GBufferPass(targets),
    overlay: new OverlayPass(overlays),
    sun: new SunPass(targets, uniforms),
    wallmarks: new WallmarkPass(targets),
    water: new WaterPass(targets, uniforms),
  };
}
