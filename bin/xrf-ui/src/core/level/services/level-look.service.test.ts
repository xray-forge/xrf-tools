import { beforeEach, describe, expect, it } from "@jest/globals";

import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import { EXrayEngine, EXrayEngineEvidence } from "@/core/ipc/types/xrf-engine-target";
import { ELevelLookSource, MONOLITH_LEVEL_BLOOM, OPENXRAY_LEVEL_BLOOM } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { LEVEL_LOOK_STORAGE_KEY } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

/** A game that leaves its bloom to the engine, as Call of Pripyat ships no console defaults at all. */
const UNSET: LevelConsoleDefaults = {
  ambientScale: null,
  bloomRadius: null,
  bloomStrength: null,
  bloomThreshold: null,
  colorGrading: null,
  hemiScale: null,
  imageExposure: null,
  imageGamma: null,
  imageSaturation: null,
  isShipped: false,
  isTonemapped: null,
  sunScale: null,
  tonemapAdaptation: null,
  tonemapAmount: null,
  tonemapLowLuminance: null,
  tonemapMiddleGray: null,
};

describe("LevelLookService", () => {
  beforeEach(() => {
    window.localStorage.clear();
    resetMockInvoke();
  });

  it("looks as the open level's game ships, and as the settings say where it ships nothing", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_console_defaults"]: mockSessionResponse({
        ambientScale: null,
        colorGrading: null,
        hemiScale: null,
        imageExposure: null,
        imageGamma: 1.2,
        imageSaturation: null,
        isShipped: true,
        isTonemapped: null,
        sunScale: 2,
        tonemapAdaptation: null,
        tonemapAmount: null,
        tonemapLowLuminance: null,
        tonemapMiddleGray: 1.5,
      }),
    });

    const { service } = mockInjectedService(LevelLookService);

    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);

    await service.open({ sessionId: "level", value: mockSelectedLevelDescription() });

    expect(service.look.lightScales.sun).toBe(2);
    expect(service.look.exposure.middleGray).toBe(1.5);
    expect(service.look.corrections.gamma).toBe(1.2);

    service.setSource(ELevelLookSource.SETTINGS);

    expect(service.look.lightScales.sun).toBe(1);
    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);
  });

  // The look and the weather follow one engine, the level's: a look reading a setting of its own disagreed with the
  // weather the level was opened with.
  it("blooms as the engine the open level was read as", async () => {
    setMockInvokeResponses({ ["plugin:levels|describe_console_defaults"]: mockSessionResponse(UNSET) });

    const { service } = mockInjectedService(LevelLookService);

    await service.open({
      sessionId: "extended",
      value: mockSelectedLevelDescription({
        engine: { engine: EXrayEngine.EXTENDED, evidence: EXrayEngineEvidence.ANOMALY_EXECUTABLES, subject: null },
      }),
    });

    expect(service.engine).toBe(EXrayEngine.EXTENDED);
    expect(service.game?.bloom).toEqual(MONOLITH_LEVEL_BLOOM);

    await service.open({ sessionId: "vanilla", value: mockSelectedLevelDescription() });

    expect(service.engine).toBe(EXrayEngine.VANILLA);
    expect(service.game?.bloom).toEqual(OPENXRAY_LEVEL_BLOOM);
  });

  it("holds no engine with no level open", async () => {
    setMockInvokeResponses({ ["plugin:levels|describe_console_defaults"]: mockSessionResponse(UNSET) });

    const { service } = mockInjectedService(LevelLookService);

    await service.open({ sessionId: "level", value: mockSelectedLevelDescription() });
    await service.open(null);

    expect(service.engine).toBeNull();
    expect(service.game).toBeNull();
  });

  it("keeps a look edited by hand over runs", () => {
    const { service } = mockInjectedService(LevelLookService);

    service.edit({ ...service.look, lightScales: { ...service.look.lightScales, sun: 3 } });

    expect(JSON.parse(window.localStorage.getItem(LEVEL_LOOK_STORAGE_KEY) ?? "{}").source).toBe(
      ELevelLookSource.CUSTOM
    );
    expect(mockInjectedService(LevelLookService).service.look.lightScales.sun).toBe(3);
  });
});
