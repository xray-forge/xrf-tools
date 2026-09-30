import { describe, expect, it, jest } from "@jest/globals";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { RendererTextures } from "#/texture/renderer-textures";
import { WeatherFader } from "#/weather/weather-fader";
import { WeatherTextures } from "#/weather/weather-textures";

const FROM: IRendererLighting = { ...DEFAULT_RENDERER_LIGHTING, waterIntensity: 0 };
const TO: IRendererLighting = { ...DEFAULT_RENDERER_LIGHTING, waterIntensity: 1 };

function createFader(isUp: boolean): WeatherFader {
  const textures: WeatherTextures = new WeatherTextures(
    new RendererTextures(
      () => {},
      () => {}
    )
  );

  jest.spyOn(textures, "isUploaded").mockReturnValue(isUp);

  return new WeatherFader(textures);
}

describe("WeatherFader", () => {
  it("shows the weather as it is while no fade runs", () => {
    expect(createFader(true).apply(0, TO)).toBe(TO);
  });

  it("runs its time from the frame the skies it fades into are up, then ends", () => {
    const fader: WeatherFader = createFader(true);

    fader.start(FROM, 1000);
    fader.ask(0);

    expect(fader.apply(0, TO).waterIntensity).toBe(0);
    expect(fader.apply(500, TO).waterIntensity).toBeCloseTo(0.5);
    expect(fader.apply(1000, TO)).toBe(TO);
    expect(fader.isFading).toBe(false);
  });

  // A sky that never uploads would hold the view on what it fades from for good.
  it("waits two seconds at most for skies that are not up", () => {
    const fader: WeatherFader = createFader(false);

    fader.start(FROM, 1000);
    fader.ask(100);

    expect(fader.apply(1500, TO).waterIntensity).toBe(0);
    expect(fader.apply(2100, TO).waterIntensity).toBe(0);
    expect(fader.apply(2600, TO).waterIntensity).toBeCloseTo(0.5);
  });

  it("forgets the fade it is stopped in", () => {
    const fader: WeatherFader = createFader(true);

    fader.start(FROM, 1000);
    fader.stop();

    expect(fader.isFading).toBe(false);
    expect(fader.held).toEqual([]);
    expect(fader.apply(0, TO)).toBe(TO);
  });
});
