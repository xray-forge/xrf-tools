import { TRendererLight } from "#/contract/scene/renderer-light";
import { IRendererLightAnimator } from "#/contract/scene/renderer-light-animator";

/**
 * The local lights of a scene, and the animations they name.
 */
export interface IRendererLights {
  lights: ReadonlyArray<TRendererLight>;
  animators: ReadonlyArray<IRendererLightAnimator>;
}
