import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { Vector3, Vector4 } from "three/webgpu";

import { IRendererTreeWind } from "#/contract/renderer-lighting";

/**
 * The sway of the trees as their shaders read it, built each frame as `FTreeVisual_setup::calculate` builds it: a wind
 * turning once every `rotation` seconds at the amplitude's length, and a wave travelling through the level at the
 * speed. Held in renderer space, where the engine's `z` is negated.
 */
export class TreeWindUniforms {
  /** The engine's `wind`: which way the trees lean, and how far, across the ground. */
  public readonly wind = uniform(new Vector3()).setGroup(renderGroup);
  /** The engine's `wave`: its direction through the level, and its phase in `w`, both over a turn. */
  public readonly wave = uniform(new Vector4()).setGroup(renderGroup);

  private trees: Nullable<IRendererTreeWind> = null;

  /** Whether the trees sway, which a shadow map drawn of them has to follow. */
  public get isSwaying(): boolean {
    return this.trees !== null && this.trees.amplitude > 0;
  }

  /**
   * @param trees - How the trees sway, or null for trees standing still.
   */
  public take(trees: Nullable<IRendererTreeWind>): void {
    this.trees = trees;
  }

  /**
   * @param time - Seconds the renderer has been running, the engine's `fTimeGlobal`.
   */
  public update(time: number): void {
    const trees: Nullable<IRendererTreeWind> = this.trees;

    if (!trees || trees.amplitude <= 0) {
      this.wind.value.set(0, 0, 0);

      return;
    }

    const turn: number = Math.PI * 2;
    const rotation: number = trees.rotation > 0 ? (turn * time) / trees.rotation : 0;
    const [x, y, z] = trees.wave;

    // `(sin, 0, cos)` normalised, then scaled: already unit, so the amplitude alone.
    this.wind.value.set(Math.sin(rotation), 0, -Math.cos(rotation)).multiplyScalar(trees.amplitude);
    this.wave.value.set(x, y, -z, time * trees.speed).divideScalar(turn);
  }
}
