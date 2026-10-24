import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container, EventBus } from "@wirestate/core";

import { SESSION_CHANGE_EVENT } from "@/core/ipc/session";
import { LEVEL_ENVIRONMENT_PART, LEVEL_SPAWN_PART, SessionChange } from "@/core/ipc/types/xrf-app";
import { EWorldWeatherPlay } from "@/core/ipc/types/xrf-world";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherCycle, mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const READ_WEATHER: string = "plugin:levels|describe_weather";
/** The level `get_level` restores, as `mockSessionResponse` names a call carrying no session. */
const SESSION: string = "fixture-session";

function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === READ_WEATHER).length;
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

/** Provisions the weather over a restored level, once its first cycle plays. */
async function openWeather(): Promise<{ container: Container; service: LevelWeatherService }> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
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

    setMockInvokeResponses({
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

    setMockInvokeResponses({
      ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
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

    setMockInvokeResponses({ [READ_WEATHER]: () => Promise.reject(new Error("cannot read")) });
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
