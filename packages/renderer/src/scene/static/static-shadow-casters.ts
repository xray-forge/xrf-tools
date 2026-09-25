import { Scene, Vector4 } from "three/webgpu";

/**
 * What the sun's shadow cascades draw: a scene a cascade of every casting batch, the cells shown to each by its box, a
 * version saying whether anything any of them draws changed, and the parts drawn plainly that cast.
 */
export interface IStaticShadowCasters {
  /** What each cascade draws. */
  readonly cascadeScenes: ReadonlyArray<Scene>;
  /**
   * What every cascade draws besides: a twin of each part drawn plainly that casts, skinned ones among them, placed
   * and skinned by three as the part is. While it holds any, a cascade draws whenever it is due, since they move.
   */
  readonly plainCasters: Scene;
  /** Bumped whenever what any batch draws changed. */
  readonly shadowVersion: number;
  /**
   * @param view - A cascade.
   * @param planes - Its box's planes, which the cells it shows are taken by.
   */
  showShadowCells(view: number, planes: ReadonlyArray<Vector4>): void;
}
