import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDERER_EXPOSURE_SETTINGS } from "@xrf/renderer";

import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import {
  DEFAULT_LEVEL_LOOK_CHOICE,
  ELevelLookSource,
  ILevelLook,
  NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  NEUTRAL_LEVEL_LIGHT_SCALES,
  resolveLevelLook,
  toGameLevelLook,
  toLevelLookChoice,
  toSettingsLevelLook,
} from "@/core/level/lib/look";

/** What Anomaly's `default_controls.ltx` sets. */
const ANOMALY: LevelConsoleDefaults = {
  ambientScale: 1,
  colorGrading: null,
  hemiScale: 1,
  imageExposure: null,
  imageGamma: null,
  imageSaturation: null,
  isShipped: true,
  isTonemapped: true,
  sunScale: 2,
  tonemapAdaptation: 10,
  tonemapAmount: 1,
  tonemapLowLuminance: 0.5,
  tonemapMiddleGray: 1.5,
};

describe("level look", () => {
  it("takes what the game ships and leaves the rest to the settings and the engine", () => {
    const look: ILevelLook = toGameLevelLook(ANOMALY, DEFAULT_RENDERER_EXPOSURE_SETTINGS);

    expect(look.exposure).toEqual({ adaptation: 10, amount: 1, isEnabled: true, lowLuminance: 0.5, middleGray: 1.5 });
    expect(look.lightScales).toEqual({ ambient: 1, hemi: 1, sun: 2 });
    expect(look.corrections).toEqual(NEUTRAL_LEVEL_IMAGE_CORRECTIONS);

    expect(
      toGameLevelLook({ ...ANOMALY, tonemapMiddleGray: null }, DEFAULT_RENDERER_EXPOSURE_SETTINGS).exposure.middleGray
    ).toBe(DEFAULT_RENDERER_EXPOSURE_SETTINGS.middleGray);
  });

  it("draws with the game's look, the settings', or its own, falling back where one is missing", () => {
    const game: ILevelLook = toGameLevelLook(ANOMALY, DEFAULT_RENDERER_EXPOSURE_SETTINGS);
    const settings: ILevelLook = toSettingsLevelLook(DEFAULT_RENDERER_EXPOSURE_SETTINGS);
    const custom: ILevelLook = { ...settings, lightScales: { ...NEUTRAL_LEVEL_LIGHT_SCALES, sun: 3 } };

    expect(resolveLevelLook(DEFAULT_LEVEL_LOOK_CHOICE, game, DEFAULT_RENDERER_EXPOSURE_SETTINGS)).toBe(game);
    expect(resolveLevelLook(DEFAULT_LEVEL_LOOK_CHOICE, null, DEFAULT_RENDERER_EXPOSURE_SETTINGS)).toEqual(settings);
    expect(
      resolveLevelLook(
        { ...DEFAULT_LEVEL_LOOK_CHOICE, source: ELevelLookSource.SETTINGS },
        game,
        DEFAULT_RENDERER_EXPOSURE_SETTINGS
      )
    ).toEqual(settings);
    expect(
      resolveLevelLook(
        { ...DEFAULT_LEVEL_LOOK_CHOICE, custom, source: ELevelLookSource.CUSTOM },
        game,
        DEFAULT_RENDERER_EXPOSURE_SETTINGS
      )
    ).toBe(custom);
  });

  it("keeps only what is well formed of a stored choice", () => {
    expect(toLevelLookChoice("junk", DEFAULT_RENDERER_EXPOSURE_SETTINGS)).toBe(DEFAULT_LEVEL_LOOK_CHOICE);
    // A custom look without its values cannot be drawn, so it is the game's.
    expect(toLevelLookChoice({ source: ELevelLookSource.CUSTOM }, DEFAULT_RENDERER_EXPOSURE_SETTINGS).source).toBe(
      ELevelLookSource.GAME
    );

    const choice = toLevelLookChoice(
      {
        custom: { lightScales: { sun: "bright" } },
        preset: "dusk",
        presets: [{ look: { corrections: { gamma: 1.2 } }, name: " dusk " }, { name: "" }, 4],
        source: ELevelLookSource.CUSTOM,
      },
      DEFAULT_RENDERER_EXPOSURE_SETTINGS
    );

    expect(choice.source).toBe(ELevelLookSource.CUSTOM);
    expect(choice.custom?.lightScales.sun).toBe(1);
    expect(choice.presets.map((it) => it.name)).toEqual(["dusk"]);
    expect(choice.presets[0]?.look.corrections.gamma).toBe(1.2);
    expect(choice.presets[0]?.look.exposure).toEqual(DEFAULT_RENDERER_EXPOSURE_SETTINGS);
  });
});
