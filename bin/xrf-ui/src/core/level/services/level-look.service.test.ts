import { beforeEach, describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDERER_EXPOSURE_SETTINGS } from "@xrf/renderer";

import { ELevelLookSource } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LEVEL_LOOK_STORAGE_KEY } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

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

    expect(service.look.exposure).toEqual(DEFAULT_RENDERER_EXPOSURE_SETTINGS);

    await service.open({ sessionId: "level", value: mockSelectedLevelDescription() });

    expect(service.look.lightScales.sun).toBe(2);
    expect(service.look.exposure.middleGray).toBe(1.5);
    expect(service.look.corrections.gamma).toBe(1.2);

    service.setSource(ELevelLookSource.SETTINGS);

    expect(service.look.lightScales.sun).toBe(1);
    expect(service.look.exposure).toEqual(DEFAULT_RENDERER_EXPOSURE_SETTINGS);
  });

  it("saves the look as a preset, picks it again after an edit, and keeps the presets over runs", () => {
    const { service } = mockInjectedService(LevelLookService);

    service.edit({ ...service.look, lightScales: { ...service.look.lightScales, sun: 3 } });
    service.savePreset("  bright  ");
    service.edit({ ...service.look, lightScales: { ...service.look.lightScales, sun: 1 } });

    expect(service.choice.preset).toBeNull();

    service.pickPreset("bright");

    expect(service.choice.preset).toBe("bright");
    expect(service.look.lightScales.sun).toBe(3);
    expect(JSON.parse(window.localStorage.getItem(LEVEL_LOOK_STORAGE_KEY) ?? "{}").presets).toHaveLength(1);

    const { service: again } = mockInjectedService(LevelLookService);

    expect(again.choice.presets.map((it) => it.name)).toEqual(["bright"]);

    again.deletePreset("bright");

    expect(again.choice.presets).toEqual([]);
    expect(again.look.lightScales.sun).toBe(3);
  });
});
