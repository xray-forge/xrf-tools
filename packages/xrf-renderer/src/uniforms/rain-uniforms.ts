import { saturate, toDirection } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { renderGroup, uniform } from "three/tsl";
import { DepthTexture, FloatType, RedFormat, RenderTarget, UniformNode, Vector3, Vector4 } from "three/webgpu";

import { IRendererRainfall } from "#/contract/renderer-rainfall";
import { RAIN_COVER_RESOLUTION, RAIN_COVER_WIDTH, RainCover } from "#/visibility/rain-cover";

/** `max_desired_items`: streaks at the heaviest rain; half as many at the lightest. */
export const RAIN_STREAKS: number = 2500;

/** `drop_max_angle`: how far the strongest wind leans the streaks from the vertical. */
const MAX_LEAN: number = (10 * Math.PI) / 180;

/** `drop_max_wind_vel`: the wind that leans them that far. */
const MAX_LEAN_WIND: number = 20;

/**
 * `wind_strength_factor / 10` with no gusts: the weather's Perlin noise is zero, so the factor is a half.
 */
const GUST: number = 0.5 / 10;

/**
 * The rain as `dxRainRender::Render` draws it: how many streaks fall, their colour and cover, the way they lean, the
 * time they fall by, and the cover over them seen from above.
 */
export class RainUniforms {
  /** What stands over the rain around the camera. */
  public readonly cover: RainCover = new RainCover();
  /** Its depth, drawn from above, beside a byte of colour three cannot draw without, never written. */
  public readonly coverTarget: RenderTarget = createCoverTarget();
  /** The depth itself, reversed: one at the height it is seen from, nought where nothing stands. */
  public readonly coverDepth: DepthTexture = this.coverTarget.depthTexture as DepthTexture;
  /** The cover's centre in `x` and `z`, its half width, and the height it is seen from. */
  public readonly window: UniformNode<"vec4", Vector4> = uniform(
    new Vector4(0, 0, RAIN_COVER_WIDTH / 2, -1e9)
  ).setGroup(renderGroup);
  /** `rain_color`, and the streaks' cover `factor / 2 + .5` in alpha. */
  public readonly color: UniformNode<"vec4", Vector4> = uniform(new Vector4()).setGroup(renderGroup);
  /** The way the rain falls, in renderer space, before each streak strays from it. */
  public readonly axis: UniformNode<"vec3", Vector3> = uniform(new Vector3(0, -1, 0)).setGroup(renderGroup);
  /** Streaks drawn, `0.5 * (1 + factor) * max_desired_items`. */
  public readonly count: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);
  /** Seconds the renderer has been running. */
  public readonly time: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /** Whether it rains at all. */
  public isFalling: boolean = false;

  /**
   * @param rain - How hard it rains now, or null for no rain.
   */
  public take(rain: Nullable<IRendererRainfall>): void {
    this.isFalling = rain !== null;

    if (!rain) {
      this.count.value = 0;

      return;
    }

    const density: number = saturate(rain.density);
    const [red, green, blue] = rain.color.map(saturate);
    const lean: number = saturate((rain.windVelocity * GUST) / MAX_LEAN_WIND);
    const pitch: number = MAX_LEAN * lean - Math.PI / 2;

    this.count.value = Math.floor(0.5 * (1 + density) * RAIN_STREAKS);
    this.color.value.set(red, green, blue, density / 2 + 0.5);

    // `axis.setHP(wind_direction, pitch)`, engine `z` negated into renderer space.
    const [x, y, z] = toDirection({ heading: rain.windDirection, pitch });

    this.axis.value.set(x, y, -z);
  }

  /**
   * @param time - Seconds the renderer has been running.
   */
  public update(time: number): void {
    this.time.value = time;
  }

  /** Points the streaks at where the cover now stands, once it is drawn there. */
  public commitCover(): void {
    const { center } = this.cover;

    this.window.value.set(center.x, center.z, RAIN_COVER_WIDTH / 2, center.y);
  }

  public dispose(): void {
    this.coverTarget.dispose();
  }
}

/** The cover's map, made as a shadow map is, so the casters' pipelines built for those draw into it. */
function createCoverTarget(): RenderTarget {
  const target: RenderTarget = new RenderTarget(RAIN_COVER_RESOLUTION, RAIN_COVER_RESOLUTION, {
    depthBuffer: true,
    format: RedFormat,
  });

  target.texture.name = "rain-cover";
  target.depthTexture = new DepthTexture(RAIN_COVER_RESOLUTION, RAIN_COVER_RESOLUTION, FloatType);
  target.depthTexture.name = "rain-cover-depth";

  return target;
}
