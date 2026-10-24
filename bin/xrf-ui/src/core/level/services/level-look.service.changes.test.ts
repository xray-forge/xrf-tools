import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus } from "@wirestate/core";

import { SESSION_CHANGE_EVENT } from "@/core/ipc/session";
import { LEVEL_CONSOLE_PART, LEVEL_SPAWN_PART, LevelConsoleDefaults, SessionChange } from "@/core/ipc/types/xrf-app";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

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
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:levels|describe_console_defaults").length;
}

afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelLookService session changes", () => {
  it("reads the console defaults again for a change naming them, and for no other part", async () => {
    let revision: number = 0;

    setMockInvokeResponses({
      ["plugin:levels|describe_console_defaults"]: () =>
        mockSessionSnapshot(mockDefaults(revision + 1), "level", revision),
    });

    const { container, service } = mockInjectedService(LevelLookService, [LevelLoadService]);

    await service.open({ sessionId: "level", revision: 0, value: mockSelectedLevelDescription() });

    expect(service.defaults.value?.sunScale).toBe(1);

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
  });
});
