import { DepthPyramidPass } from "#/pass/depth-pyramid-pass";
import { GBufferLatePass } from "#/pass/gbuffer-late-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticLateCullPass } from "#/pass/static-late-cull-pass";
import { StaticCull } from "#/scene/static/static-cull";

/** The passes a frame culls what the depth hides by, by their place in it. */
export interface IOcclusionFramePasses {
  readonly lateCull: IRendererPass;
  readonly gbufferLate: IRendererPass;
  readonly pyramid: IRendererPass;
}

/**
 * The second phase of the static draws' occlusion cull and the depth pyramid both phases read: the static draws the
 * first phase's depth hid culled again and drawn into the G-buffer, then its depth reduced for the next frame.
 *
 * @param targets - What the passes draw into.
 * @param cull - What culls the static draws.
 * @returns The passes, by their place in the frame.
 */
export function createOcclusionFramePasses(targets: RendererTargets, cull: StaticCull): IOcclusionFramePasses {
  return {
    gbufferLate: new GBufferLatePass(targets, cull),
    lateCull: new StaticLateCullPass(targets, cull),
    pyramid: new DepthPyramidPass(targets, cull),
  };
}
