import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { LevelConsoleDefaults, SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngine, EXrayEngineChoice, EXrayEngineEvidence } from "@xrf/ipc/types/xrf-engine-target";

import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { MONOLITH_LEVEL_BLOOM, OPENXRAY_LEVEL_BLOOM } from "@/core/level/lib/look/level-look";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage, getLocalStorageValue, LEVEL_LOOK_STORAGE_KEY } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";

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

async function mockOpenLook(
  defaults: LevelConsoleDefaults,
  level: SelectedLevelDescription = mockSelectedLevelDescription()
): Promise<{ container: Container; service: LevelLookService }> {
  mockIpcCommands({
    [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: mockSessionResponse(defaults),
    [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(level),
  });

  const container: Container = mockContainer([LevelLoadService, LevelLookService]).provision();
  const service: LevelLookService = container.get(LevelLookService);

  await waitFor(() => expect(service.defaults.isReady).toBe(true));

  return { container, service };
}

describe("LevelLookService", () => {
  beforeEach(() => {
    clearLocalStorage();
    resetMockInvoke();
  });

  it("looks as the open level's game ships, and as the settings say where it ships nothing", async () => {
    const { container, service } = await mockOpenLook({
      ...UNSET,
      imageGamma: 1.2,
      isShipped: true,
      sunScale: 2,
      tonemapMiddleGray: 1.5,
    });

    expect(service.look.lightScales.sun).toBe(2);
    expect(service.look.exposure.middleGray).toBe(1.5);
    expect(service.look.corrections.gamma).toBe(1.2);

    service.setSource(ELevelLookSource.SETTINGS);

    expect(service.look.lightScales.sun).toBe(1);
    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);

    container.deprovision();
  });

  it("looks as the settings say before a level opens", () => {
    const { service } = mockInjectedService(LevelLookService, [LevelLoadService]);

    expect(service.look.exposure).toEqual(DEFAULT_RENDER_EXPOSURE_SETTINGS);
  });

  // The look and the weather follow one engine, the level's: a look reading a setting of its own disagreed with the
  // weather the level was opened with.
  it("blooms as the engine the open level was read as", async () => {
    const { container, service } = await mockOpenLook(
      UNSET,
      mockSelectedLevelDescription({
        engine: { engine: EXrayEngine.EXTENDED, evidence: EXrayEngineEvidence.ANOMALY_EXECUTABLES, subject: null },
      })
    );

    expect(service.engine).toBe(EXrayEngine.EXTENDED);
    expect(service.game?.bloom).toEqual(MONOLITH_LEVEL_BLOOM);

    const level: SelectedLevelDescription = mockSelectedLevelDescription();

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: mockSessionResponse(UNSET),
      [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(level),
    });
    await container.get(LevelLoadService).load({
      engine: EXrayEngineChoice.VANILLA,
      isDltx: false,
      roots: level.roots,
      source: level.source,
    });
    await waitFor(() => expect(service.defaults.isReady).toBe(true));

    expect(service.engine).toBe(EXrayEngine.VANILLA);
    expect(service.game?.bloom).toEqual(OPENXRAY_LEVEL_BLOOM);

    container.deprovision();
  });

  it("keeps a look edited by hand over runs", () => {
    const { service } = mockInjectedService(LevelLookService, [LevelLoadService]);

    service.edit({ ...service.look, lightScales: { ...service.look.lightScales, sun: 3 } });

    expect(JSON.parse(getLocalStorageValue(LEVEL_LOOK_STORAGE_KEY) ?? "{}").source).toBe(ELevelLookSource.CUSTOM);
    expect(mockInjectedService(LevelLookService, [LevelLoadService]).service.look.lightScales.sun).toBe(3);
  });

  // A read that failed is not a game that ships none: the look says which, while both fall back to the engine's.
  it("holds a failed read of the defaults as failed, not as none shipped", async () => {
    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
      [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: () => {
        throw new Error("the level's archive went");
      },
    });

    const container: Container = mockContainer([LevelLoadService, LevelLookService]).provision();
    const service: LevelLookService = container.get(LevelLookService);

    await waitFor(() => expect(service.defaults.isFailed).toBe(true));
    expect(service.defaults.error?.message).toBe("the level's archive went");
    expect(service.game).toBeNull();

    container.deprovision();
  });

  it("reads the defaults of a level as the level opens, with no viewport to draw it, and drops them as it closes", async () => {
    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
      [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: mockSessionResponse(UNSET),
    });

    const container: Container = mockContainer([LevelLoadService, LevelLookService]).provision();
    const service: LevelLookService = container.get(LevelLookService);

    await waitFor(() => expect(service.defaults.value).toEqual(UNSET));
    expect(service.engine).toBe(EXrayEngine.VANILLA);

    container.get(LevelLoadService).clear();

    expect(service.defaults.isIdle).toBe(true);
    expect(service.engine).toBeNull();
    expect(service.game).toBeNull();

    container.deprovision();
  });
});
