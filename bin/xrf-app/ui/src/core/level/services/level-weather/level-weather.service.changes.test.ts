import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container, EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { LEVEL_ENVIRONMENT_PART, LEVEL_SPAWN_PART } from "@xrf/ipc/types/xrf-app-levels";
import { EWorldWeatherPlay } from "@xrf/ipc/types/xrf-world";

import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { mockLevelWeatherCycle, mockLevelWeatherDescription } from "@/core/level/fixtures/weather.mocks";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";

const READ_WEATHER: string = ELevelsCommand.DESCRIBE_WEATHER;
/** The level `restore_level` restores, as `mockSessionResponse` names a call carrying no session. */
const SESSION: string = "fixture-session";

function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === READ_WEATHER).length;
}

/** Provisions the weather over a restored level, once its first cycle plays. */
async function openWeather(): Promise<{ container: Container; service: LevelWeatherService }> {
  mockIpcCommands({
    [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
    [READ_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
  });

  const container: Container = mockContainer([LevelLoadService, LevelWeatherService]).provision();
  const service: LevelWeatherService = container.get(LevelWeatherService);

  await waitFor(() => expect(service.cycle.value?.name).toBe("default_clear"));

  return { container, service };
}

function notify(container: Container, change: SessionChange): void {
  container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
}

afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelWeatherService changes", () => {
  it("reads the weather and the cycle played again once a change lets the environment go, keeping what plays", async () => {
    const { container, service } = await openWeather();
    const before: number = countReads();
    const edited = mockLevelWeatherCycle({ file: "environment\\weathers\\default_clear_edited.ltx" });

    mockIpcCommands({
      [READ_WEATHER]: mockSessionSnapshot(mockLevelWeatherDescription({ offered: [edited] }), SESSION, 1),
    });
    notify(container, {
      sessionId: SESSION,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_ENVIRONMENT_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(countReads() - before).toBe(1);
    expect(service.description.value?.offered).toEqual([edited]);
    expect(service.cycle.value).toEqual(edited);
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_clear" });

    container.deprovision();
  });

  // The catch-up after an open derives the level while its weather is first read: the weather plays the read that
  // overtook the first, rather than falling back on the keyframe set by hand.
  it("plays the weather read again while it was first being read", async () => {
    const answers: Array<() => void> = [];

    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
      [READ_WEATHER]: (args?: Record<string, unknown>) =>
        new Promise((resolve) =>
          answers.push(() =>
            resolve(mockSessionSnapshot(mockLevelWeatherDescription(), args?.sessionId as string, answers.length))
          )
        ),
    });

    const container: Container = mockContainer([LevelLoadService, LevelWeatherService]).provision();
    const service: LevelWeatherService = container.get(LevelWeatherService);

    await waitFor(() => expect(answers).toHaveLength(1));
    notify(container, {
      sessionId: SESSION,
      revision: 1,
      root: { kind: "derived" },
      staleParts: [],
      isViewRenewed: true,
    });
    await flush();
    answers[0]();
    await flush();
    answers[1]();

    await waitFor(() => expect(service.cycle.value?.name).toBe("default_clear"));
    expect(service.isManual).toBe(false);

    container.deprovision();
  });

  it("plays on as read where the weather fails to be read again, saying nothing failed", async () => {
    const { container, service } = await openWeather();

    mockIpcCommands({ [READ_WEATHER]: () => Promise.reject(new Error("cannot read")) });
    notify(container, {
      sessionId: SESSION,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_ENVIRONMENT_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(service.description.error?.message).toBe("cannot read");
    expect(service.failure).toBeNull();
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_clear" });

    container.deprovision();
  });

  it("reads nothing again for another level's change, or one that left the environment be", async () => {
    const { container } = await openWeather();
    const before: number = countReads();

    notify(container, {
      sessionId: "another",
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_ENVIRONMENT_PART],
      isViewRenewed: false,
    });
    notify(container, {
      sessionId: SESSION,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(countReads()).toBe(before);

    container.deprovision();
  });
});
