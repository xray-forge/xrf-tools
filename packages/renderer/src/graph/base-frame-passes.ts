import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `Base`'s frame, in R4's order: the G-buffer, wall marks into its albedo, the sun, combine with its fog and tonemap,
 * then blended surfaces and the helpers over the tonemapped frame.
 *
 * @param targets - What the passes draw into.
 * @param uniforms - What their shaders read.
 * @param overlays - The helpers drawn last.
 * @returns The passes, in frame order, before the frame is presented.
 */
export function createBaseFramePasses(
  targets: RendererTargets,
  uniforms: RendererUniforms,
  overlays: RendererOverlays
): Array<IRendererPass> {
  return [
    new GBufferPass(targets),
    new WallmarkPass(targets),
    new SunPass(targets, uniforms),
    new CombinePass(targets, uniforms),
    new ForwardPass(targets),
    new OverlayPass(overlays),
  ];
}
