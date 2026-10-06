import { describe, expect, it } from "@jest/globals";

import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import {
  DEFAULT_LEVEL_LOOK_CHOICE,
  ELevelLookSource,
  ILevelLook,
  MONOLITH_LEVEL_BLOOM,
  NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  NEUTRAL_LEVEL_LIGHT_SCALES,
  NO_LEVEL_BLOOM,
  OPENXRAY_LEVEL_BLOOM,
  resolveLevelLook,
  toGameLevelLook,
  toLevelLookChoice,
  toSettingsLevelLook,
} from "@/core/level/lib/look";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";

/** What Anomaly's `default_controls.ltx` sets. */
const ANOMALY: LevelConsoleDefaults = {
  ambientScale: 1,
  bloomRadius: 1,
  bloomStrength: 0.05,
  bloomThreshold: 0,
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
    const look: ILevelLook = toGameLevelLook(ANOMALY, DEFAULT_RENDER_EXPOSURE_SETTINGS, EXrayEngine.EXTENDED);

    expect(look.bloom).toEqual({ isEnabled: true, radius: 1, strength: 0.05, threshold: 0 });
    expect(look.exposure).toEqual({ adaptation: 10, amount: 1, isEnabled: true, lowLuminance: 0.5, middleGray: 1.5 });
    expect(look.lightScales).toEqual({ ambient: 1, hemi: 1, sun: 2 });
    expect(look.corrections).toEqual(NEUTRAL_LEVEL_IMAGE_CORRECTIONS);

    expect(
      toGameLevelLook({ ...ANOMALY, tonemapMiddleGray: null }, DEFAULT_RENDER_EXPOSURE_SETTINGS, EXrayEngine.EXTENDED)
        .exposure.middleGray
    ).toBe(DEFAULT_RENDER_EXPOSURE_SETTINGS.middleGray);
  });

  it("blooms as the level's engine does where the game sets no bloom, and the settings bloom nothing", () => {
    const unset: LevelConsoleDefaults = { ...ANOMALY, bloomRadius: null, bloomStrength: null, bloomThreshold: null };

    expect(toGameLevelLook(unset, DEFAULT_RENDER_EXPOSURE_SETTINGS, EXrayEngine.VANILLA).bloom).toEqual(
      OPENXRAY_LEVEL_BLOOM
    );
    expect(toGameLevelLook(unset, DEFAULT_RENDER_EXPOSURE_SETTINGS, EXrayEngine.EXTENDED).bloom).toEqual(
      MONOLITH_LEVEL_BLOOM
    );
    expect(toSettingsLevelLook(DEFAULT_RENDER_EXPOSURE_SETTINGS).bloom).toEqual(NO_LEVEL_BLOOM);
  });

  it("draws with the game's look, the settings', or its own, falling back where one is missing", () => {
    const game: ILevelLook = toGameLevelLook(ANOMALY, DEFAULT_RENDER_EXPOSURE_SETTINGS, EXrayEngine.EXTENDED);
    const settings: ILevelLook = toSettingsLevelLook(DEFAULT_RENDER_EXPOSURE_SETTINGS);
    const custom: ILevelLook = { ...settings, lightScales: { ...NEUTRAL_LEVEL_LIGHT_SCALES, sun: 3 } };

    expect(resolveLevelLook(DEFAULT_LEVEL_LOOK_CHOICE, game, DEFAULT_RENDER_EXPOSURE_SETTINGS)).toBe(game);
    expect(resolveLevelLook(DEFAULT_LEVEL_LOOK_CHOICE, null, DEFAULT_RENDER_EXPOSURE_SETTINGS)).toEqual(settings);
    expect(
      resolveLevelLook(
        { ...DEFAULT_LEVEL_LOOK_CHOICE, source: ELevelLookSource.SETTINGS },
        game,
        DEFAULT_RENDER_EXPOSURE_SETTINGS
      )
    ).toEqual(settings);
    expect(
      resolveLevelLook(
        { ...DEFAULT_LEVEL_LOOK_CHOICE, custom, source: ELevelLookSource.CUSTOM },
        game,
        DEFAULT_RENDER_EXPOSURE_SETTINGS
      )
    ).toBe(custom);
  });

  it("keeps only what is well formed of a stored choice", () => {
    expect(toLevelLookChoice("junk", DEFAULT_RENDER_EXPOSURE_SETTINGS)).toBe(DEFAULT_LEVEL_LOOK_CHOICE);
    // A custom look without its values cannot be drawn, so it is the game's.
    expect(toLevelLookChoice({ source: ELevelLookSource.CUSTOM }, DEFAULT_RENDER_EXPOSURE_SETTINGS).source).toBe(
      ELevelLookSource.GAME
    );

    const choice = toLevelLookChoice(
      {
        custom: { lightScales: { sun: "bright" } },
        source: ELevelLookSource.CUSTOM,
      },
      DEFAULT_RENDER_EXPOSURE_SETTINGS
    );

    expect(choice.source).toBe(ELevelLookSource.CUSTOM);
    expect(choice.custom?.lightScales.sun).toBe(1);
    expect(choice.custom?.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);
    // A look kept before the bloom blooms nothing, as it drew.
    expect(choice.custom?.bloom).toEqual(NO_LEVEL_BLOOM);
  });
});
