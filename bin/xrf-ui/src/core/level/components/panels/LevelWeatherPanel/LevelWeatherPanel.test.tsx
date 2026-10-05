import { afterEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EEnvironmentRule, WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { EWorldWeatherPlay } from "@/core/ipc/types/xrf-world";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService, LevelViewService, LevelWeatherService } from "@/core/level/services";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { InvokeMap, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockLevelWeatherCycle,
  mockLevelWeatherDescription,
  mockWorldWeatherReport,
} from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelWeatherPanel } from "./LevelWeatherPanel";

async function renderPanel(responses: InvokeMap = {}): Promise<RenderResult & { weather: LevelWeatherService }> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
    ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
      mockLevelWeatherCycle({ name: cycle.name })
    ),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(
      mockLevelWeatherDescription({
        cycles: [{ file: "", findings: 3, keyframes: 24, kind: "cycle", name: "default_rain" }],
        effects: [mockLevelWeatherCycle({ kind: "effect", name: "fx_blowout" })],
        offered: [
          mockLevelWeatherCycle({
            findings: [
              {
                file: "environment\\weathers\\default_clear.ltx",
                key: "fog_color",
                message: "Weather [12:00:00] writes 2 of fog_color's 3 components",
                rule: EEnvironmentRule.ENGINE,
                section: "12:00:00",
              },
            ],
          }),
        ],
      })
    ),
    ...responses,
  });

  const container: Container = mockContainer([LevelLoadService, LevelViewService, LevelWeatherService]);
  const load: LevelLoadService = container.get(LevelLoadService);
  const weather: LevelWeatherService = container.get(LevelWeatherService);

  const result: RenderResult = renderWithProviders(<LevelWeatherPanel />, { container });

  // Mounting provisions the container, and that restores the level the backend holds.
  await waitFor(() => expect(load.isReady).toBe(true));
  await act(() => weather.open(load.level.value?.selected ?? null));

  return { ...result, weather };
}

afterEach(() => {
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelWeatherPanel", () => {
  it("plays the level's own cycle first, marked, then the game's others, with the playing one's findings", async () => {
    const { getByRole, getByTestId } = await renderPanel();

    expect(getByRole("button", { name: "Weather" })).toHaveAttribute("aria-pressed", "true");
    expect(getByRole("combobox", { name: "Cycle" }).textContent).toBe("default_clear");

    await userEvent.click(getByRole("combobox", { name: "Cycle" }));

    expect(getByRole("listbox").textContent).toMatch(/default_clear · level.*default_rain.*3 findings/);
    expect(getByTestId("level-weather-findings-section").textContent).toContain("writes 2 of fog_color");
    expect(getByTestId("level-weather-clock-section").textContent).toContain("12:00:00");
  });

  it("plays a cycle picked from the list, lights by hand on asking, and by the weather again on a pick", async () => {
    const { getByRole, getByTestId, weather } = await renderPanel();

    await userEvent.click(getByRole("combobox", { name: "Cycle" }));
    await userEvent.click(getByRole("option", { name: /default_rain/ }));
    await waitFor(() => expect(weather.cycle?.name).toBe("default_rain"));

    await userEvent.click(getByRole("button", { name: "Manual" }));

    expect(weather.source).toBe(ELevelWeatherSource.MANUAL);
    await waitFor(() => expect(weather.weather?.kind).toBe(EWorldWeatherPlay.KEYFRAME));
    expect(getByTestId("level-weather-play-section").textContent).toContain("Seeded from");
    expect(getByRole("checkbox", { name: "Dynamic sun" })).toBeDisabled();

    await userEvent.click(getByRole("combobox", { name: "Cycle" }));
    await userEvent.click(getByRole("option", { name: /default_clear/ }));

    await waitFor(() => expect(weather.source).toBe(ELevelWeatherSource.WEATHER));
    expect(weather.cycle?.name).toBe("default_clear");
  });

  it("goes back to the weather from the keyframe set by hand", async () => {
    const { getByRole, weather } = await renderPanel();

    await userEvent.click(getByRole("button", { name: "Manual" }));
    await userEvent.click(getByRole("button", { name: "Back to weather" }));

    expect(weather.source).toBe(ELevelWeatherSource.WEATHER);
  });

  it("runs the clock from its button, and offers the dynamic sun on vanilla alone", async () => {
    const { getByRole, weather } = await renderPanel();

    await userEvent.click(getByRole("button", { name: "Play" }));

    expect(weather.control.isPaused).toBe(false);
    expect(getByRole("checkbox", { name: "Dynamic sun" })).not.toBeChecked();
  });

  it("says why the weather cannot light a level whose weather does not read, and keeps it manual", async () => {
    const { getByRole, getByTestId } = await renderPanel({
      ["plugin:levels|read_level_weather"]: () => Promise.reject(new Error("The configs are gone")),
    });

    expect(getByRole("button", { name: "Weather" })).toBeDisabled();
    expect(getByRole("button", { name: "Manual" })).toHaveAttribute("aria-pressed", "true");
    expect(getByTestId("level-weather-play-section").textContent).toContain("The configs are gone");
  });

  it("stands the sun by the table on extended where the game has one", async () => {
    const { getByTestId, queryByRole } = await renderPanel({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({ engine: EXrayEngine.EXTENDED, sunTable: [] })
      ),
    });

    expect(queryByRole("checkbox", { name: "Dynamic sun" })).not.toBeInTheDocument();
    expect(getByTestId("level-weather-clock-section").textContent).toContain("sun table");
  });

  // A game made for OpenXRay, opened on extended, has no table: its keyframes stand the sun, as on vanilla.
  it("stands the sun by the keyframes on extended where the game has no table", async () => {
    const { getByRole, getByTestId } = await renderPanel({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({ engine: EXrayEngine.EXTENDED, sunTable: null })
      ),
    });

    expect(getByRole("checkbox", { name: "Dynamic sun" })).toBeInTheDocument();
    expect(getByTestId("level-weather-clock-section").textContent).not.toContain("sun table");
  });

  it("plays an effect over the cycle, and says what is left of the one playing with a way to end it", async () => {
    const { getByRole, getByTestId, weather } = await renderPanel();

    await userEvent.click(getByRole("combobox", { name: "Effect" }));
    await userEvent.click(getByRole("option", { name: /fx_blowout/ }));

    expect(weather.effect).toEqual({ name: "fx_blowout" });

    act(() =>
      weather.noteReport(mockWorldWeatherReport({ effect: { name: "fx_blowout", remaining: 125 }, time: 43_300 }))
    );

    await waitFor(() =>
      expect(getByTestId("level-weather-play-section").textContent).toContain("00:02:05 of game time left")
    );

    await userEvent.click(getByRole("combobox", { name: "Effect" }));
    await userEvent.click(getByRole("option", { name: "None" }));

    expect(weather.effect).toEqual({ name: null });
  });

  it("says what ambient effect plays near the camera and asks for one at once", async () => {
    const { getByRole, getByTestId, weather } = await renderPanel();

    act(() =>
      weather.noteReport(
        mockWorldWeatherReport({
          ambient: {
            effect: { name: "effect_6", particles: "nature\\fog_stormy_01", remaining: 4 },
            isIndoors: false,
            played: 1,
            wait: 20,
          },
        })
      )
    );

    await waitFor(() => expect(getByTestId("level-weather-ambient-section").textContent).toContain("effect_6"));

    await userEvent.click(getByRole("button", { name: "Play one now" }));

    expect(weather.ambientPlays).toBe(1);
  });

  it("says how many of the level's own overrides there are, and how many reach the camera", async () => {
    const { getByTestId, weather } = await renderPanel({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({
          modifiers: [
            {
              ambient: { x: 0, y: 0, z: 0 },
              farPlane: 100,
              fogColor: { x: 0, y: 0, z: 0 },
              fogDensity: 0,
              hemiColor: { x: 0, y: 0, z: 0 },
              position: { x: 0, y: 0, z: 0 },
              power: 1,
              radius: 10,
              skyColor: { x: 0, y: 0, z: 0 },
              useFlags: 1,
            },
          ],
        })
      ),
    });

    function section(): string {
      return getByTestId("level-weather-modifiers-section").textContent ?? "";
    }

    expect(section()).toMatch(/Volumes\s*1.*Around the camera\s*0/);

    act(() => weather.noteReport(mockWorldWeatherReport({ modifiers: 1, weight: 1 })));

    await waitFor(() => expect(section()).toMatch(/Around the camera\s*1/));
  });

  it("stands empty until a level is open", () => {
    const { getByTestId } = renderWithProviders(<LevelWeatherPanel />, {
      container: mockContainer([LevelLoadService, LevelViewService, LevelWeatherService]),
    });

    expect(getByTestId("level-weather-panel").textContent).toContain("No level open");
  });
});
