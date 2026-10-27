import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container, EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { LEVEL_CONSOLE_PART, LEVEL_SPAWN_PART, LevelConsoleDefaults } from "@xrf/ipc/types/xrf-app-levels";

import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";

/** Console defaults setting only the sun's scale, which tells one read from the next. */
function mockDefaults(sunScale: number): LevelConsoleDefaults {
  return {
    ambientScale: null,
    bloomRadius: null,
    bloomStrength: null,
    bloomThreshold: null,
    colorGrading: null,
    hemiScale: null,
    imageExposure: null,
    imageGamma: null,
    imageSaturation: null,
    isShipped: true,
    isTonemapped: null,
    sunScale,
    tonemapAdaptation: null,
    tonemapAmount: null,
    tonemapLowLuminance: null,
    tonemapMiddleGray: null,
  };
}

function countReads(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS).length;
}

afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelLookService session changes", () => {
  it("reads the console defaults again for a change naming them, and for no other part", async () => {
    let revision: number = 0;

    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: mockSessionSnapshot(mockSelectedLevelDescription(), "level", 0),
      [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: () =>
        mockSessionSnapshot(mockDefaults(revision + 1), "level", revision),
    });

    // The level the backend still holds, restored as the viewer restores it.
    const container: Container = mockContainer([LevelLoadService, LevelLookService]).provision();
    const service: LevelLookService = container.get(LevelLookService);

    await waitFor(() => expect(service.defaults.value?.sunScale).toBe(1));

    const reads: number = countReads();

    function notify(change: SessionChange): void {
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
    }

    revision = 1;
    notify({
      sessionId: "level",
      revision,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });

    expect(countReads()).toBe(reads);

    revision = 2;
    notify({
      sessionId: "level",
      revision,
      root: { kind: "kept" },
      staleParts: [LEVEL_CONSOLE_PART],
      isViewRenewed: false,
    });

    await waitFor(() => expect(service.defaults.value?.sunScale).toBe(3));
    expect(countReads()).toBe(reads + 1);

    container.deprovision();
  });
});
