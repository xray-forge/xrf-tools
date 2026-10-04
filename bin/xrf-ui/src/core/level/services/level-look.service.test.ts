import { beforeEach, describe, expect, it } from "@jest/globals";

import { ELevelLookSource } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
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

    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);

    await service.open({ sessionId: "level", value: mockSelectedLevelDescription() });

    expect(service.look.lightScales.sun).toBe(2);
    expect(service.look.exposure.middleGray).toBe(1.5);
    expect(service.look.corrections.gamma).toBe(1.2);

    service.setSource(ELevelLookSource.SETTINGS);

    expect(service.look.lightScales.sun).toBe(1);
    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);
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
