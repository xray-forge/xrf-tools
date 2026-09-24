import { CombinePass } from "#/pass/combine-pass";
import { DepthPyramidPass } from "#/pass/depth-pyramid-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferLatePass } from "#/pass/gbuffer-late-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCullPass } from "#/pass/static-cull-pass";
import { StaticLateCullPass } from "#/pass/static-late-cull-pass";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { StaticCull } from "#/scene/static/static-cull";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `Base`'s frame, in R4's order: the static draws culled, the G-buffer, the static draws its depth hid culled again
 * and drawn into it, its depth reduced for the next frame, wall marks into its albedo, the sun, combine with its fog and
 * tonemap, then blended surfaces and the helpers over the tonemapped frame.
 *
 * @param targets - What the passes draw into.
 * @param uniforms - What their shaders read.
 * @param overlays - The helpers drawn last.
 * @param cull - What culls the static draws.
 * @returns The passes, in frame order, before the frame is presented.
 */
export function createBaseFramePasses(
  targets: RendererTargets,
  uniforms: RendererUniforms,
  overlays: RendererOverlays,
  cull: StaticCull
): Array<IRendererPass> {
  return [
    new StaticCullPass(cull),
    new GBufferPass(targets),
    new StaticLateCullPass(targets, cull),
    new GBufferLatePass(targets, cull),
    new DepthPyramidPass(targets, cull),
    new WallmarkPass(targets),
    new SunPass(targets, uniforms),
    new CombinePass(targets, uniforms),
    new ForwardPass(targets),
    new OverlayPass(overlays),
  ];
}
