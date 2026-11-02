import { BoundAction, comparer, makeObservable, reaction } from "@wirestate/mobx";
import { EWorldWeatherPlay, WorldWeatherControl, WorldWeatherPlay } from "@xrf/ipc/types/xrf-world";
import { Maybe, Nullable } from "@xrf/types";

import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** The level's weather as its viewport plays it: what plays, how its clock runs, and the effects played over it. */
export class LevelRenderWeather {
  /**
   * @param context - The viewport the weather plays in.
   * @param weatherService - The weather asked for, and where it stands.
   */
  public constructor(
    private readonly context: ILevelRenderContext,
    private readonly weatherService: LevelWeatherService
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * Plays the level's weather on from a time of day, which is where its clock stands from then on.
   *
   * @param time - Seconds since midnight.
   */
  @BoundAction()
  public seek(time: number): void {
    this.weatherService.seekTo(time);
    this.context.getViewport()?.seekWeather(time);
  }

  /**
   * @param name - The weather effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  @BoundAction()
  public playEffect(name: Nullable<string>): void {
    this.context.getViewport()?.playWeatherEffect(name);
  }

  /** Plays a weather ambient effect near the camera at once, ending the one playing; none plays indoors. */
  @BoundAction()
  public playAmbientEffect(): void {
    this.context.getViewport()?.playAmbientEffect();
  }

  /**
   * Plays the level's weather in a viewport just attached, and how it plays, as either changes.
   *
   * @param viewport - The viewport just attached.
   * @returns What stops watching.
   */
  public start(viewport: NativeViewport): Array<() => void> {
    const { weatherService } = this;

    return [
      reaction(
        () => weatherService.weather,
        (play: Nullable<WorldWeatherPlay>, previous: Maybe<Nullable<WorldWeatherPlay>>) => {
          viewport.playWeather(play ?? { kind: EWorldWeatherPlay.NONE }, weatherService.transition);

          // The renderer's clock starts at noon: a level's first weather plays on from where its own clock stands.
          if (play && !previous) {
            viewport.seekWeather(weatherService.time);
          }
        },
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): WorldWeatherControl => ({
          ...weatherService.control,
          // The keyframe set by hand stands its sun by its own angles.
          isDynamicSun: weatherService.control.isDynamicSun && !weatherService.isManual,
        }),
        (control: WorldWeatherControl) => viewport.setWeatherControl(control),
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }
}
