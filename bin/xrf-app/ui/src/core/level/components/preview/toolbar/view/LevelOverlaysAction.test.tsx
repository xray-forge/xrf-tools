import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelOverlaysAction } from "@/core/level/components/preview/toolbar/view/LevelOverlaysAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { SettingsRendererDisplay } from "@/core/settings/components/SettingsDialog/SettingsRenderSection/SettingsRendererDisplay";
import { clearLocalStorage } from "@/core/storage";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelOverlaysAction", () => {
  it("names the overlays shown, and times the passes from the readouts", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isAxesVisible", true);

    const { getByRole, findByRole } = renderLevelView(<LevelOverlaysAction />, descriptor);

    expect(getByRole("button", { name: "Overlays" })).toHaveAccessibleDescription("Showing axes, readouts");

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Grid" })).not.toBeChecked();
  });

  // The marker says where the level's sunlight comes from, which the weather's own sun in the sky already shows by day.
  it("marks the sun among the overlays, off unless asked for", async () => {
    const { getByRole, findByRole, view } = renderLevelView(<LevelOverlaysAction />, mockLevelViewContainer());

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Sun" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Sun" }));

    expect(view.options.isSunMarked).toBe(true);
  });

  // One switch, offered where the readout is and where the settings are: either place sets it, and both show it.
  it("times the passes from its readouts and from the settings alike", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole, queryByRole } = renderLevelView(
      <>
        <LevelOverlaysAction />
        <SettingsRendererDisplay />
      </>,
      descriptor
    );
    const { renderSettings } = descriptor;

    expect(renderSettings.isGpuTimed).toBe(false);

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await userEvent.click(
      within(await findByRole("dialog", { name: "Overlays" })).getByRole("checkbox", { name: "GPU time per pass" })
    );

    expect(renderSettings.isGpuTimed).toBe(true);

    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(queryByRole("dialog", { name: "Overlays" })).not.toBeInTheDocument());

    expect(getByRole("checkbox", { name: "GPU time per pass" })).toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "GPU time per pass" }));
    await userEvent.click(getByRole("button", { name: "Overlays" }));

    expect(renderSettings.isGpuTimed).toBe(false);
    expect(
      within(await findByRole("dialog", { name: "Overlays" })).getByRole("checkbox", { name: "GPU time per pass" })
    ).not.toBeChecked();
  });
});
