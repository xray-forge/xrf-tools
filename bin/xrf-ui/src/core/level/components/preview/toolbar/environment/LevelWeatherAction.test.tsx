import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { LevelWeatherAction } from "@/core/level/components/preview/toolbar/environment/LevelWeatherAction";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

async function renderAction(): Promise<{ weather: LevelWeatherService } & ReturnType<typeof renderWithProviders>> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelLookService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelRenderService,
  ]);
  const load: LevelLoadService = container.get(LevelLoadService);
  const weather: LevelWeatherService = container.get(LevelWeatherService);

  await load.restore();
  await weather.open(load.level.value?.selected ?? null);

  return { ...renderWithProviders(<LevelWeatherAction />, { container }), weather };
}

afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelWeatherAction", () => {
  it("says what lights the level and when, and switches it to the keyframe set by hand", async () => {
    const { getByRole, findByRole, weather } = await renderAction();

    expect(getByRole("button", { name: "Weather" })).toHaveAccessibleDescription("default_clear at 12:00");

    await userEvent.click(getByRole("button", { name: "Weather" }));
    await findByRole("dialog", { name: "Weather" });
    await userEvent.click(getByRole("button", { name: "Manual" }));

    expect(weather.source).toBe(ELevelWeatherSource.MANUAL);
    expect(getByRole("dialog", { name: "Weather" }).textContent).toContain("Play the weather");
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
