import { RendererTargets } from "#/pass/renderer-targets";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { SceneRain } from "#/scene/rain/scene-rain";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { SceneThunder } from "#/scene/thunder/scene-thunder";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** What `Base`'s passes are made over. */
export interface IBaseFramePassesInput {
  /** What the passes draw into. */
  targets: RendererTargets;
  /** What their shaders read. */
  uniforms: RendererUniforms;
  /** The helpers drawn last. */
  overlays: RendererOverlays;
  /** What culls the static draws. */
  cull: StaticCull;
  /** What casts shadows, which covers the rain too. */
  casters: IStaticShadowCasters;
  /** The rain's draws. */
  rain: SceneRain;
  /** The bolts' draws. */
  thunder: SceneThunder;
}
