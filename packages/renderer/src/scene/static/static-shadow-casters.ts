import { Scene } from "three/webgpu";

import { PlainShadowCasters } from "#/scene/static/plain-shadow-casters";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

/**
 * What the shadow views draw, the sun's cascades and the lights' faces: a scene a view of every casting batch, each
 * drawing what its view's cull kept of it, the parts drawn plainly that cast, and where what any of them draws changed.
 */
export interface IStaticShadowCasters {
  /** What each shadow view draws: its still and its swaying scene. */
  readonly shadowScenes: ReadonlyArray<Scene>;
  /** What each shadow view draws of the casters that stand still, which a light's face keeps while the rest sway. */
  readonly stillShadowScenes: ReadonlyArray<Scene>;
  /** And of those that sway with the wind, which a face draws again over what it kept. */
  readonly swayingShadowScenes: ReadonlyArray<Scene>;
  /**
   * What every shadow view draws besides: a twin of each part drawn plainly that casts, skinned ones among them, placed
   * and skinned by three as the part is, each noted in the changes where it stands.
   */
  readonly plainCasters: PlainShadowCasters;
  /** Where what the shadow views draw changed, and what of it sways or moves: what a kept shadow is drawn again by. */
  readonly shadowChanges: StaticShadowChanges;
}
