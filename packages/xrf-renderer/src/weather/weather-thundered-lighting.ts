import { TRendererColor } from "#/contract/renderer-color";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererThunderSettings } from "#/contract/weather/renderer-thunder-settings";
import { IWeatherThunderFlash } from "#/weather/weather-thunder-flash";

/**
 * `CEffect_Thunderbolt::OnFrame` over the mix: the strike's colour added to the sky (clamped), the sun and the fog,
 * and the sun turned to come from the bolt.
 *
 * @param lighting - What the weather lights the frame with.
 * @param flash - The strike under way.
 * @param settings - How much of its colour each is lit by.
 * @returns The frame lit by the strike, the bolt drawn.
 */
export function toThunderedLighting(
  lighting: IRendererLighting,
  flash: IWeatherThunderFlash,
  settings: IRendererThunderSettings
): IRendererLighting {
  const { color, direction } = flash;

  function added(base: TRendererColor, by: number): TRendererColor {
    return [base[0] + color[0] * by, base[1] + color[1] * by, base[2] + color[2] * by];
  }

  const sky: TRendererColor = added(lighting.sky.color, settings.skyColor);

  return {
    ...lighting,
    fog: lighting.fog ? { ...lighting.fog, color: added(lighting.fog.color, settings.fogColor) } : null,
    sky: { ...lighting.sky, color: [clamp(sky[0]), clamp(sky[1]), clamp(sky[2])] },
    sunColor: added(lighting.sunColor, settings.sunColor),
    // Engine `z` negated into renderer space.
    sunDirection: [direction[0], direction[1], -direction[2]],
    thunderbolt: flash.strike,
  };
}

function clamp(value: number): number {
  return Math.min(Math.max(value, 0), 1);
}
