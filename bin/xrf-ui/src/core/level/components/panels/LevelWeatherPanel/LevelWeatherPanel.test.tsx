import { afterEach, describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EEnvironmentRule, WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService, LevelWeatherService } from "@/core/level/services";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { InvokeMap, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherCycle, mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
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

  const container: Container = mockContainer([LevelLoadService, LevelWeatherService]);
  const load: LevelLoadService = container.get(LevelLoadService);
  const weather: LevelWeatherService = container.get(LevelWeatherService);

  await load.restore();
  await weather.open(load.level.value?.selected ?? null);

  return { ...renderWithProviders(<LevelWeatherPanel />, { container }), weather };
}

afterEach(() => {
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelWeatherPanel", () => {
  it("plays the level's own cycle first, marked, then the game's others, with the playing one's findings", async () => {
    const { getByRole, getByTestId } = await renderPanel();
    const cycles: HTMLElement = getByTestId("level-weather-cycles-section");

    expect(getByRole("button", { name: "Weather" })).toHaveAttribute("aria-pressed", "true");
    expect(cycles.textContent).toMatch(/default_clear.*Level.*default_rain.*3 findings/);
    expect(getByTestId("level-weather-findings-section").textContent).toContain("writes 2 of fog_color");
    expect(getByTestId("level-weather-clock-section").textContent).toContain("12:00:00");
  });

  it("plays a cycle picked from the list, and lights by hand on asking", async () => {
    const { getByRole, weather } = await renderPanel();

    await userEvent.click(getByRole("button", { name: /default_rain/ }));
    await waitFor(() => expect(weather.cycle?.name).toBe("default_rain"));

    await userEvent.click(getByRole("button", { name: "Manual" }));

    expect(weather.source).toBe(ELevelWeatherSource.MANUAL);
    expect(weather.weather).toBeNull();
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
    expect(getByTestId("level-weather-source-section").textContent).toContain("The configs are gone");
  });

  it("stands the sun by the table on extended", async () => {
    const { getByTestId, queryByRole } = await renderPanel({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({ engine: EXrayEngine.EXTENDED })
      ),
    });

    expect(queryByRole("checkbox", { name: "Dynamic sun" })).not.toBeInTheDocument();
    expect(getByTestId("level-weather-sun-section").textContent).toContain("sun table");
  });

  it("plays an effect over the cycle, and says what is left of the one playing with a way to end it", async () => {
    const { getByRole, getByTestId, weather } = await renderPanel();

    await userEvent.click(getByRole("button", { name: /fx_blowout/ }));

    expect(weather.effect).toEqual({ name: "fx_blowout" });

    weather.noteReport({
      effect: { name: "fx_blowout", remaining: 125 },
      keyframes: null,
      modifiers: 0,
      time: 43_300,
      weight: 0.5,
    });

    await waitFor(() =>
      expect(getByTestId("level-weather-effects-section").textContent).toContain("00:02:05 of game time left")
    );

    await userEvent.click(getByRole("button", { name: "Stop" }));

    expect(weather.effect).toEqual({ name: null });
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

    weather.noteReport({ effect: null, keyframes: [0, 1], modifiers: 1, time: 43_200, weight: 1 });

    await waitFor(() => expect(section()).toMatch(/Around the camera\s*1/));
  });

  it("stands empty until a level is open", () => {
    const { getByTestId } = renderWithProviders(<LevelWeatherPanel />, {
      container: mockContainer([LevelLoadService, LevelWeatherService]),
    });

    expect(getByTestId("level-weather-panel").textContent).toContain("No level open");
  });
});
