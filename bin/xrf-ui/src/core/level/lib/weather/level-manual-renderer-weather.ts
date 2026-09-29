import { IRendererWeather, IRendererWeatherKeyframe } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelTextureReference, LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelManualWeather, toLevelManualKeyframe } from "@/core/level/lib/weather/level-manual-weather";
import {
  listLevelRainTextures,
  toLevelRendererTextures,
  toLevelRendererWeatherBase,
  toLevelRendererWeatherEngine,
} from "@/core/level/lib/weather/level-renderer-weather";

/** What a hand-set keyframe the renderer plays is built from. */
export interface ILevelManualRendererWeatherInput {
  manual: ILevelManualWeather;
  /** The open level's weather, whose effects, modifiers and rain it plays with; null where it does not read. */
  description: Nullable<LevelWeatherDescription>;
  /** The engine target, for a level whose weather does not read. */
  engine: XrayEngine;
  /** Roots the level was opened in. */
  roots: XrayRoots;
  /** The keyframe's own textures, resolved as the level resolves its own. */
  located: ReadonlyArray<LevelTextureReference>;
}

/**
 * A keyframe set by hand as a cycle of one, which the renderer plays as it plays any: it stands its sun by its own
 * angles on either engine, so no sun table goes with it, and an effect played over it that stands no sun of its own
 * stands it where the keyframe does.
 *
 * @param input - The keyframe, the level's weather and where the textures are read from.
 * @returns What the renderer plays.
 */
export async function toLevelManualRendererWeather(input: ILevelManualRendererWeatherInput): Promise<IRendererWeather> {
  const { manual, description, engine, roots, located } = input;
  const keyframe: IRendererWeatherKeyframe = toLevelManualKeyframe(manual, 0);
  const base = description
    ? toLevelRendererWeatherBase(description)
    : { effects: {}, engine: toLevelRendererWeatherEngine(engine), modifiers: [], rain: null };

  return {
    ...base,
    effects: Object.fromEntries(
      Object.entries(base.effects).map(([name, keyframes]) => [
        name,
        keyframes.map((it: IRendererWeatherKeyframe) =>
          it.sunDirection ? it : { ...it, sunDirection: keyframe.sunDirection }
        ),
      ])
    ),
    keyframes: [keyframe],
    sunTable: null,
    textures: await toLevelRendererTextures(roots, [
      ...located,
      ...(description ? description.effects.flatMap((it: LevelWeatherCycle) => it.textures) : []),
      ...(description ? listLevelRainTextures(description.rain) : []),
    ]),
  };
}
