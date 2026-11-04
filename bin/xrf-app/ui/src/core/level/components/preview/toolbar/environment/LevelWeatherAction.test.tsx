import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { resetMockInvoke } from "@xrf/ipc/testing";

import { LevelWeatherAction } from "@/core/level/components/preview/toolbar/environment/LevelWeatherAction";
import {
  mockLevelViewContainer,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { clearLocalStorage } from "@/core/storage";

function renderAction(): Promise<TLevelViewRenderResult> {
  return renderOpenLevelView(<LevelWeatherAction />, mockLevelViewContainer());
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

  it("shows the time of day as the panel does: the cycle's keyframes marked, and its speed beneath", async () => {
    const { getByRole, findByRole, weather } = await renderAction();

    await userEvent.click(getByRole("button", { name: "Weather" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Weather" });
    const marks: NodeListOf<Element> = dialog.querySelectorAll(".MuiSlider-mark");

    expect(getByRole("slider", { name: "Time of day" })).toBeInTheDocument();
    expect(marks.length).toBeGreaterThan(0);
    expect(marks).toHaveLength(weather.cycle.value?.keyframes.length ?? -1);

    getByRole("slider", { name: "Speed" }).focus();
    await userEvent.keyboard("{End}");

    expect(weather.control.factor).toBeGreaterThan(12);
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
