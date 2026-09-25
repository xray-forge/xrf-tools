import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { Vector3, Vector4 } from "three/webgpu";

import { IRendererGrassSwing, IRendererGrassWind } from "#/contract/renderer-lighting";

/** The first wave's direction through the level, over a turn (`CDetailManager::hw_Render`). */
const FIRST_WAVE: readonly [number, number, number] = [1 / 5, 1 / 7, 1 / 3];

/** The second wave's, the same components in another order. */
const SECOND_WAVE: readonly [number, number, number] = [1 / 3, 1 / 7, 1 / 5];

/**
 * The sway of the grass as its shaders read it, built each frame as `CDetailManager::hw_Render` builds it: two winds
 * turning at their own rates and a wave running through the level, the normal and fast swings mixed by the wind's
 * strength. Held in the engine's own space, where the grass is swayed.
 */
export class GrassWindUniforms {
  /** `dir1`: the first wave's lean across the ground. */
  public readonly wind1 = uniform(new Vector3()).setGroup(renderGroup);
  /** `dir2`: the second's. */
  public readonly wind2 = uniform(new Vector3()).setGroup(renderGroup);
  /** The first wave: its direction, and its phase in `w`, both over a turn. */
  public readonly wave1 = uniform(new Vector4()).setGroup(renderGroup);
  public readonly wave2 = uniform(new Vector4()).setGroup(renderGroup);
  /** The same the frame before, where a tuft's vertex stood then for the motion it wrote. */
  public readonly previousWind1 = uniform(new Vector3()).setGroup(renderGroup);
  public readonly previousWind2 = uniform(new Vector3()).setGroup(renderGroup);
  public readonly previousWave1 = uniform(new Vector4()).setGroup(renderGroup);
  public readonly previousWave2 = uniform(new Vector4()).setGroup(renderGroup);

  private grass: Nullable<IRendererGrassWind> = null;

  /**
   * @param grass - How the grass sways, or null for grass standing still.
   */
  public take(grass: Nullable<IRendererGrassWind>): void {
    this.grass = grass;
  }

  /**
   * @param time - Seconds the renderer has been running, the engine's `fTimeGlobal`.
   */
  public update(time: number): void {
    this.previousWind1.value.copy(this.wind1.value);
    this.previousWind2.value.copy(this.wind2.value);
    this.previousWave1.value.copy(this.wave1.value);
    this.previousWave2.value.copy(this.wave2.value);

    if (!this.grass) {
      this.wind1.value.set(0, 0, 0);
      this.wind2.value.set(0, 0, 0);

      return;
    }

    const swing: IRendererGrassSwing = mixSwing(this.grass);
    const turn: number = Math.PI * 2;
    const first: number = swing.rot1 > 0 ? (turn * time) / swing.rot1 : 0;
    const second: number = swing.rot2 > 0 ? (turn * time) / swing.rot2 : 0;
    const phase: number = time * swing.speed;

    // `(sin, 0, cos)` normalised, then scaled: already unit, so the amplitude alone.
    this.wind1.value.set(Math.sin(first), 0, Math.cos(first)).multiplyScalar(swing.amp1);
    this.wind2.value.set(Math.sin(second), 0, Math.cos(second)).multiplyScalar(swing.amp2);
    this.wave1.value.set(...FIRST_WAVE, phase).divideScalar(turn);
    this.wave2.value.set(...SECOND_WAVE, phase).divideScalar(turn);
  }
}

/** The swing between the normal and the fast one, `swing_current.lerp`. */
function mixSwing({ strength, normal, fast }: IRendererGrassWind): IRendererGrassSwing {
  function mix(from: number, to: number): number {
    return from + (to - from) * strength;
  }

  return {
    amp1: mix(normal.amp1, fast.amp1),
    amp2: mix(normal.amp2, fast.amp2),
    rot1: mix(normal.rot1, fast.rot1),
    rot2: mix(normal.rot2, fast.rot2),
    speed: mix(normal.speed, fast.speed),
  };
}
