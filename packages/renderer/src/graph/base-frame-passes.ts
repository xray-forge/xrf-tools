import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCullPass } from "#/pass/static-cull-pass";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
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
  readonly forward: IRendererPass;
  readonly overlay: OverlayPass;
}

/**
 * `Base`'s frame, in R4's order: the static draws culled, the G-buffer, wall marks into its albedo, the sun, combine
 * with its fog and tonemap, then blended surfaces and the helpers over the tonemapped frame.
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
    forward: new ForwardPass(targets),
    gbuffer: new GBufferPass(targets),
    overlay: new OverlayPass(overlays),
    sun: new SunPass(targets, uniforms),
    wallmarks: new WallmarkPass(targets),
  };
}
