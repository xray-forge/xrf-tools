import { Scene, Vector4 } from "three/webgpu";

/**
 * What the sun's shadow cascades draw of the static draws: a scene a cascade of every casting batch, the cells shown
 * to each by its box, and a version saying whether anything any of them draws changed.
 */
export interface IStaticShadowCasters {
  /** What each cascade draws. */
  readonly cascadeScenes: ReadonlyArray<Scene>;
  /** Bumped whenever what any batch draws changed. */
  readonly shadowVersion: number;
  /**
   * @param view - A cascade.
   * @param planes - Its box's planes, which the cells it shows are taken by.
   */
  showShadowCells(view: number, planes: ReadonlyArray<Vector4>): void;
}
