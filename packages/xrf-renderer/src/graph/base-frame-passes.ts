import { IBaseFramePassesInput } from "#/graph/base-frame-passes-input";
import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferEarlyPass } from "#/pass/gbuffer-early-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { RainCoverPass } from "#/pass/rain-cover-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { StaticCullPass } from "#/pass/static-cull-pass";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { WeatherPass } from "#/pass/weather-pass";
import { WetPass } from "#/pass/wet-pass";

/** `Base`'s passes every frame draws, by their place in it. */
export interface IBaseFramePasses {
  readonly cull: IRendererPass;
  readonly gbufferEarly: IRendererPass;
  readonly gbuffer: IRendererPass;
  readonly wallmarks: IRendererPass;
  readonly sun: IRendererPass;
  readonly combine: CombinePass;
  readonly forward: IRendererPass;
  readonly rainCover: IRendererPass;
  readonly wet: IRendererPass;
  readonly rain: IRendererPass;
  readonly thunder: IRendererPass;
  readonly overlay: OverlayPass;
}

/**
 * `Base`'s frame, in R4's order: the static draws culled, the G-buffer (the static draws, then the plain ones), wall
 * marks into its albedo, the rain wetting it, the sun, combine with its fog and tonemap, then the blended surfaces over
 * the tonemapped frame, the rain and the bolt striking over them, and the helpers over it all.
 *
 * @param input - What the passes are made over.
 * @returns The passes, by their place in the frame.
 */
export function createBaseFramePasses(input: IBaseFramePassesInput): IBaseFramePasses {
  const { targets, uniforms, overlays, cull, casters, rain, thunder } = input;

  return {
    combine: new CombinePass(targets, uniforms),
    cull: new StaticCullPass(cull),
    forward: new ForwardPass(targets),
    gbuffer: new GBufferPass(targets),
    gbufferEarly: new GBufferEarlyPass(targets, cull),
    overlay: new OverlayPass(overlays, targets.composite),
    rain: new WeatherPass({ name: "rain", source: rain, targets }),
    rainCover: new RainCoverPass(casters, cull, uniforms.rain),
    sun: new SunPass(targets, uniforms),
    thunder: new WeatherPass({ name: "thunder", source: thunder, targets }),
    wallmarks: new WallmarkPass(targets),
    wet: new WetPass(targets, uniforms),
  };
}
