import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { LevelWeatherAction } from "@/core/level/components/preview/LevelWeatherAction";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService, LevelWeatherService } from "@/core/level/services";
import { mockLevelTextureReference, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

async function renderAction(): Promise<{ weather: LevelWeatherService } & ReturnType<typeof renderWithProviders>> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    ["plugin:levels|resolve_level_textures"]: mockSessionResponse(({ references }: { references: Array<string> }) =>
      references.map((reference: string) => mockLevelTextureReference(reference))
    ),
  });

  const container: Container = mockContainer([LevelLoadService, LevelWeatherService]);
  const load: LevelLoadService = container.get(LevelLoadService);
  const weather: LevelWeatherService = container.get(LevelWeatherService);

  await load.restore();
  await weather.open(load.level.value?.selected ?? null);

  return { ...renderWithProviders(<LevelWeatherAction />, { container }), weather };
}

afterEach(() => {
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelWeatherAction", () => {
  it("says what lights the level and when, and switches it to the keyframe set by hand", async () => {
    const { getByRole, findByRole, weather } = await renderAction();

    expect(getByRole("button", { name: "Weather" })).toHaveAccessibleDescription("default_clear at 12:00");

    await userEvent.click(getByRole("button", { name: "Weather" }));
    await findByRole("dialog", { name: "Weather" });
    await userEvent.click(getByRole("button", { name: "Manual" }));

    expect(weather.source).toBe(ELevelWeatherSource.MANUAL);
    expect(getByRole("dialog", { name: "Weather" }).textContent).toContain("Back to weather");
  });

  it("copies the keyframe on screen as a weather config section", async () => {
    const writeText = jest.fn<(text: string) => Promise<void>>(() => Promise.resolve());
    const { getByRole, findByRole } = await renderAction();

    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });

    await userEvent.click(getByRole("button", { name: "Weather" }));
    await findByRole("dialog", { name: "Weather" });
    await userEvent.click(getByRole("button", { name: "Copy as LTX" }));

    expect(writeText.mock.calls[0]?.[0]).toMatch(/^\[12:00:00\]\nsky_texture +=/);
    await findByRole("button", { name: "Copied" });
  });
});
