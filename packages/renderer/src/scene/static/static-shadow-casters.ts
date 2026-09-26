import { Scene, Vector4 } from "three/webgpu";

import { PlainShadowCasters } from "#/scene/static/plain-shadow-casters";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

/**
 * What the shadow views draw, the sun's cascades and the lights' faces: a scene a view of every casting batch, the
 * cells shown to each by its frustum, the parts drawn plainly that cast, and where what any of them draws changed.
 */
export interface IStaticShadowCasters {
  /** What each shadow view draws. */
  readonly shadowScenes: ReadonlyArray<Scene>;
  /**
   * What every shadow view draws besides: a twin of each part drawn plainly that casts, skinned ones among them, placed
   * and skinned by three as the part is, each noted in the changes where it stands.
   */
  readonly plainCasters: PlainShadowCasters;
  /** Where what the shadow views draw changed, and what of it sways or moves: what a kept shadow is drawn again by. */
  readonly shadowChanges: StaticShadowChanges;
  /**
   * @param view - A shadow view.
   * @param planes - Its frustum's planes, which the cells it shows are taken by.
   */
  showShadowCells(view: number, planes: ReadonlyArray<Vector4>): void;
}
