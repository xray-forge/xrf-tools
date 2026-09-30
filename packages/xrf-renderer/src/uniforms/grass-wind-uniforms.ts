import { mix } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { UniformNode, Vector3, Vector4 } from "three/webgpu";

import { IRendererGrassSwing } from "#/contract/renderer-grass-swing";
import { IRendererGrassWind } from "#/contract/renderer-grass-wind";

/** The first wave's direction through the level, over a turn (`CDetailManager::hw_Render`). */
const FIRST_WAVE: readonly [number, number, number] = [1 / 5, 1 / 7, 1 / 3];

/** The second wave's, the same components in another order. */
const SECOND_WAVE: readonly [number, number, number] = [1 / 3, 1 / 7, 1 / 5];

/** What a frame's step is taken as where the time ran back or leapt past a second, as `hw_Render` takes it. */
const STEP_FALLBACK: number = 0.03;

/**
 * The sway of the grass as its shaders read it, built each frame as `CDetailManager::hw_Render` builds it: two winds
 * turning at their own rates and a wave running through the level, the normal and fast swings mixed by the wind's
 * strength, each advanced by the frame's step so a change of strength turns them no faster at once. Held in the
 * engine's own space, where the grass is swayed.
 */
export class GrassWindUniforms {
  /** `dir1`: the first wave's lean across the ground. */
  public readonly wind1: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  /** `dir2`: the second's. */
  public readonly wind2: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  /** The first wave: its direction, and its phase in `w`, both over a turn. */
  public readonly wave1: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);
  public readonly wave2: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);
  /** The same the frame before, where a tuft's vertex stood then for the motion it wrote. */
  public readonly previousWind1: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly previousWind2: UniformNode<"vec3", Vector3> = uniform(new Vector3()).setGroup(renderGroup);
  public readonly previousWave1: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);
  public readonly previousWave2: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);

  private grass: Nullable<IRendererGrassWind> = null;
  /** `m_time_rot_1`, `m_time_rot_2` and `m_time_pos`: how far each wind has turned and the waves have run. */
  private firstTurn: number = 0;
  private secondTurn: number = 0;
  private phase: number = 0;
  /** `m_global_time_old`: the time the last frame was at. */
  private time: Nullable<number> = null;
  /** `swing_current`, mixed each frame. */
  private readonly swing: IRendererGrassSwing = { amp1: 0, amp2: 0, rot1: 0, rot2: 0, speed: 0 };

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
    const elapsed: number = time - (this.time ?? time);
    const step: number = elapsed < 0 || elapsed > 1 ? STEP_FALLBACK : elapsed;

    this.time = time;
    this.previousWind1.value.copy(this.wind1.value);
    this.previousWind2.value.copy(this.wind2.value);
    this.previousWave1.value.copy(this.wave1.value);
    this.previousWave2.value.copy(this.wave2.value);

    if (!this.grass) {
      this.wind1.value.set(0, 0, 0);
      this.wind2.value.set(0, 0, 0);

      return;
    }

    const swing: IRendererGrassSwing = mixSwing(this.grass, this.swing);
    const turn: number = Math.PI * 2;

    this.firstTurn += swing.rot1 > 0 ? (turn * step) / swing.rot1 : 0;
    this.secondTurn += swing.rot2 > 0 ? (turn * step) / swing.rot2 : 0;
    this.phase += step * swing.speed;
    // `(sin, 0, cos)` normalised, then scaled: already unit, so the amplitude alone.
    this.wind1.value.set(Math.sin(this.firstTurn), 0, Math.cos(this.firstTurn)).multiplyScalar(swing.amp1);
    this.wind2.value.set(Math.sin(this.secondTurn), 0, Math.cos(this.secondTurn)).multiplyScalar(swing.amp2);
    this.wave1.value.set(...FIRST_WAVE, this.phase).divideScalar(turn);
    this.wave2.value.set(...SECOND_WAVE, this.phase).divideScalar(turn);
  }
}

/**
 * @param wind - How the grass sways.
 * @param out - Where the swing is written.
 * @returns The swing between the normal and the fast one, `swing_current.lerp`.
 */
function mixSwing(wind: IRendererGrassWind, out: IRendererGrassSwing): IRendererGrassSwing {
  const { strength, normal, fast } = wind;

  out.amp1 = mix(normal.amp1, fast.amp1, strength);
  out.amp2 = mix(normal.amp2, fast.amp2, strength);
  out.rot1 = mix(normal.rot1, fast.rot1, strength);
  out.rot2 = mix(normal.rot2, fast.rot2, strength);
  out.speed = mix(normal.speed, fast.speed, strength);

  return out;
}
