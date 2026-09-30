import { addVectors, clamp, EPS_L, mix, saturate, scaleVector, toDirection, toHeadingPitch, wrap } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { toRendererVector } from "#/contract/renderer-space";
import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererLightAnimator } from "#/contract/scene/renderer-light-animator";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IRendererThunderSettings } from "#/contract/weather/renderer-thunder-settings";
import { IRendererThunderbolt } from "#/contract/weather/renderer-thunderbolt";
import { toInterpolatedColor } from "#/lighting/light-animator";
import { IWeatherMix } from "#/weather/weather-mix";
import { IWeatherThunderFlash } from "#/weather/weather-thunder-flash";
import { IWeatherThunderInput } from "#/weather/weather-thunder-input";

/** `CEffect_Thunderbolt::MAX_DIST_FACTOR`: the farthest a bolt strikes, of the far plane. */
const MAX_DISTANCE: number = 0.95;

/** `EPS_L`: real seconds a strike followed at once waits after the last one ends. */
const SECOND_DELAY: number = EPS_L;

/** The longest real step a strike takes at once, so a view shown again after a while does not skip one. */
const LONGEST_STEP: number = 1;

/** A bolt of a collection, by name. */
interface IWeatherBolt {
  name: string;
  bolt: IRendererThunderbolt;
}

/** A strike under way. */
interface IWeatherStrike {
  bolt: IWeatherBolt;
  /** Real seconds it lasts, and has lasted. */
  life: number;
  time: number;
  /** `current_xform`'s rows: its axes scaled by its length, in engine space. */
  axes: readonly [TRendererVector, TRendererVector, TRendererVector];
  /** Where it strikes from, its top. */
  position: TRendererVector;
  /** `lightning_center`, halfway down it. */
  center: TRendererVector;
  /** `lightning_size`, its length down to the ground. */
  size: number;
  /** From it towards the view. */
  direction: TRendererVector;
}

/**
 * `CEffect_Thunderbolt`: a strike every period while the weather names a collection, a random bolt of it each time,
 * across the sky from the sun, at the far plane, down to the ground; and, while it strikes, the colour it lights the
 * frame with. It runs on real time, as the engine's global time does, so bolts strike on a paused clock too.
 */
export class WeatherThunder {
  private readonly random: () => number;
  /** Whether the weather struck last frame, `bEnabled`. */
  private isEnabled: boolean = false;
  /** Real seconds the next strike is due at. */
  private next: number = 0;
  private strike: Nullable<IWeatherStrike> = null;
  private advancedAt: Nullable<number> = null;

  /**
   * @param random - A number in `[0, 1)` each call, `Random.randF`.
   */
  public constructor(random: () => number = Math.random) {
    this.random = random;
  }

  /** Forgets any strike and when the next was due, as a weather taken afresh does. */
  public reset(): void {
    this.isEnabled = false;
    this.strike = null;
    this.advancedAt = null;
  }

  /**
   * `OnFrame`: schedules a strike as the weather starts to name a collection, strikes when one is due, and plays the
   * strike under way.
   *
   * @param input - The weather mixed now and where the view stands.
   * @returns What the strike does to the frame, or null while none is under way.
   */
  public advance(input: IWeatherThunderInput): Nullable<IWeatherThunderFlash> {
    const { thunder, mix, now } = input;
    const step: number = this.advancedAt === null ? 0 : clamp(now - this.advancedAt, 0, LONGEST_STEP);
    const palette: ReadonlyArray<IWeatherBolt> = input.isEnabled ? WeatherThunder.listPalette(thunder, mix) : [];
    const isEnabled: boolean = palette.length > 0 && thunder.settings !== null;

    this.advancedAt = now;

    if (this.isEnabled !== isEnabled) {
      const period: number = mix.thunderboltPeriod;

      this.isEnabled = isEnabled;
      this.next = now + period + this.between(-period * 0.5, period * 0.5);
    } else if (isEnabled && thunder.settings && now > this.next && !this.strike) {
      this.strike = this.bolt({ ...input, palette, settings: thunder.settings });
    }

    const { strike } = this;

    if (!strike) {
      return null;
    }

    // The engine goes idle before it steps the time, and lights this frame all the same.
    const isOver: boolean = strike.time > strike.life;

    strike.time += step;

    if (isOver) {
      this.strike = null;
    }

    return this.toFlash(strike, thunder);
  }

  /** `Bolt`: where the strike stands and how long it lasts, and when the next is due. */
  private bolt(
    input: IWeatherThunderInput & { palette: ReadonlyArray<IWeatherBolt>; settings: IRendererThunderSettings }
  ): IWeatherStrike {
    const { mix, now, palette, settings, view } = input;
    const lasting: number = mix.thunderboltDuration;
    const life: number = lasting + this.between(-lasting * 0.5, lasting * 0.5);
    const bolt: IWeatherBolt = palette[Math.min(Math.floor(this.random() * palette.length), palette.length - 1)];
    const sunHeading: number = toHeadingPitch(mix.sunDirection).heading;
    const far: number = mix.farPlane;
    const period: number = mix.thunderboltPeriod;
    const altitude: number = this.between(settings.altitude[0], settings.altitude[1]);
    const longitude: number = this.between(
      sunHeading - settings.deltaLongitude + Math.PI,
      sunHeading + settings.deltaLongitude + Math.PI
    );
    const distance: number = this.between(far * settings.minDistance, far * MAX_DISTANCE);
    const toward: TRendererVector = toDirection({ heading: longitude, pitch: altitude });
    const position: TRendererVector = addVectors(view, scaleVector(toward, distance));
    const deviation: TRendererVector = [
      this.between(-settings.tilt, settings.tilt),
      this.between(0, Math.PI * 2),
      this.between(-settings.tilt, settings.tilt),
    ];
    // `setXYZi`, which is `setHPB(-y, -x, -z)`; the light falls down the matrix's second axis.
    const [i, j, k] = toRotationRows(-deviation[1], -deviation[0], -deviation[2]);
    const down: TRendererVector = scaleVector(j, -1);
    const size: number = toGround(position, down, far * 2);

    this.next =
      this.random() < settings.secondProbability
        ? now + lasting + SECOND_DELAY
        : now + period + this.between(-period * 0.3, period * 0.3);

    return {
      axes: [scaleVector(i, size), scaleVector(j, size), scaleVector(k, size)],
      bolt,
      center: addVectors(position, scaleVector(down, size * 0.5)),
      direction: scaleVector(toward, -1),
      life,
      position,
      size,
      time: 0,
    };
  }

  private toFlash(strike: IWeatherStrike, thunder: IRendererThunder): IWeatherThunderFlash {
    const { life, size } = strike;
    const [i, j, k] = strike.axes;
    const { bolt, name } = strike.bolt;
    const progress: number = life > 0 ? strike.time / life : 1;
    const phase: number = saturate(1.5 * progress);
    const animator: Maybe<IRendererLightAnimator> = bolt.color === null ? undefined : thunder.animators[bolt.color];
    const color: Array<number> = [0, 0, 0];

    // `CalculateRGB` with the frame rate set to the frame count: the whole animation over the strike's life.
    if (animator) {
      toInterpolatedColor(animator, Math.floor(wrap(progress, 1) * animator.frameCount), color);
    }

    // The top glow's opacity lights both, as `dxThunderboltRender` has it.
    const opacity: number = bolt.top.opacity * phase;

    return {
      color: [saturate(color[0] / 255), saturate(color[1] / 255), saturate(color[2] / 255)],
      direction: strike.direction,
      strike: {
        // The model's `z` is negated into renderer space as its mesh is, so its third axis turns about too.
        axes: [toRendererVector(i), toRendererVector(j), scaleVector(toRendererVector(k), -1)],
        center: {
          extent: [bolt.center.radius[0] * size, bolt.center.radius[1] * size],
          opacity,
          position: toRendererVector(strike.center),
        },
        bolt: name,
        position: toRendererVector(strike.position),
        shift: phase > 0.5 ? Math.min(Math.floor(this.random() * 2), 1) * 0.5 : phase * 0.5,
        top: {
          extent: [bolt.top.radius[0] * size, bolt.top.radius[1] * size],
          opacity,
          position: toRendererVector(strike.position),
        },
      },
    };
  }

  /** `Random.randF(min, max)`. */
  private between(min: number, max: number): number {
    return mix(min, max, this.random());
  }

  /** The bolts of the collection the weather names now that the game has; none for no collection. */
  private static listPalette(thunder: IRendererThunder, mix: IWeatherMix): ReadonlyArray<IWeatherBolt> {
    const names: Maybe<ReadonlyArray<string>> = mix.thunderboltCollection
      ? thunder.collections[mix.thunderboltCollection]
      : undefined;

    return (names ?? []).flatMap((name: string) => {
      const bolt: Maybe<IRendererThunderbolt> = thunder.bolts[name];

      return bolt ? [{ bolt, name }] : [];
    });
  }
}

/** `Fmatrix::setHPB`'s first three rows. */
function toRotationRows(h: number, p: number, b: number): readonly [TRendererVector, TRendererVector, TRendererVector] {
  const [sh, ch, sp, cp, sb, cb] = [Math.sin(h), Math.cos(h), Math.sin(p), Math.cos(p), Math.sin(b), Math.cos(b)];
  const [cc, cs, sc, ss] = [ch * cb, ch * sb, sh * cb, sh * sb];

  return [
    [cc - sp * ss, -cp * sb, sp * cs + sc],
    [sp * sc + cs, cp * cb, ss - sp * cc],
    [-cp * sh, sp, cp * ch],
  ];
}

/**
 * `RayPick` where the level's own geometry is not at hand: how far down the bolt reaches the ground plane, `y = 0`,
 * within a range, or the range where it never does.
 */
function toGround(from: TRendererVector, down: TRendererVector, range: number): number {
  if (Math.abs(down[1]) < 1e-6) {
    return range;
  }

  const distance: number = -from[1] / down[1];

  return distance >= 0 && distance <= range ? distance : range;
}
