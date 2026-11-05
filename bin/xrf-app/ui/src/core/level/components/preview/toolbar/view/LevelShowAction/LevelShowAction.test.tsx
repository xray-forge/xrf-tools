import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";

import { LevelShowAction } from "@/core/level/components/preview/toolbar/view/LevelShowAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { SettingsRendererDisplay } from "@/core/settings/components/SettingsDialog/SettingsRenderSection/SettingsRendererDisplay";
import { clearLocalStorage } from "@/core/storage";

async function openShow(rendered: TLevelViewRenderResult): Promise<HTMLElement> {
  await userEvent.click(rendered.getByRole("button", { name: "Show" }));

  return rendered.findByRole("dialog", { name: "Show" });
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelShowAction", () => {
  it("says what it shows without being opened: the overlays, the helpers and how many kinds are drawn", () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isAxesVisible", true);
    descriptor.view.setOption("isSpawnedItems", false);

    const { getByRole } = renderLevelView(<LevelShowAction />, descriptor);

    expect(getByRole("button", { name: "Show" })).toHaveAccessibleDescription(
      expect.stringMatching(/^Axes, readouts, no helpers, particles, \d+ of \d+ kinds drawn$/)
    );
  });

  it("turns the particles, the campfires and the ambient effects over by their own switches", async () => {
    const rendered: TLevelViewRenderResult = renderLevelView(<LevelShowAction />, mockLevelViewContainer());
    const dialog: HTMLElement = await openShow(rendered);

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Particles" }));
    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Campfires lit" }));
    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Ambient effects" }));

    expect(rendered.view.options).toEqual({
      ...DEFAULT_LEVEL_VIEW_OPTIONS,
      isAmbientEffectPlayed: !DEFAULT_LEVEL_VIEW_OPTIONS.isAmbientEffectPlayed,
      isCampfireLit: !DEFAULT_LEVEL_VIEW_OPTIONS.isCampfireLit,
      isParticled: !DEFAULT_LEVEL_VIEW_OPTIONS.isParticled,
    });
    expect(rendered.getByRole("button", { name: "Show", hidden: true })).toHaveAccessibleDescription(
      expect.stringContaining("no particles")
    );
  });

  it("turns an overlay over, and times the passes for the readouts as the settings do", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const rendered: TLevelViewRenderResult = renderLevelView(
      <>
        <LevelShowAction />
        <SettingsRendererDisplay />
      </>,
      descriptor
    );
    const dialog: HTMLElement = await openShow(rendered);

    expect(within(dialog).getByRole("checkbox", { name: "Sun" })).not.toBeChecked();

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Sun" }));
    await userEvent.click(within(dialog).getByRole("checkbox", { name: "GPU time per pass" }));

    expect(rendered.view.options.isSunMarked).toBe(true);
    expect(descriptor.renderSettings.isGpuTimed).toBe(true);

    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(rendered.queryByRole("dialog", { name: "Show" })).not.toBeInTheDocument());

    // One switch, offered where the readouts are and where the settings are.
    expect(rendered.getByRole("checkbox", { name: "GPU time per pass" })).toBeChecked();
  });

  it("draws or hides each kind, and marks it by its helper once the helpers are on", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isSpawnedItems", false);

    const rendered: TLevelViewRenderResult = renderLevelView(<LevelShowAction />, descriptor);
    const dialog: HTMLElement = await openShow(rendered);

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Items shown" }));
    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Lamps shown" }));

    expect(rendered.view.options.isSpawnedItems).toBe(true);
    expect(rendered.view.options.isSpawnedLamps).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSpawnedLamps);
    // A kind drawn as nothing has its helper alone, which waits on the helpers' switch.
    expect(within(dialog).queryByRole("checkbox", { name: "Restrictors shown" })).toBeNull();
    expect(within(dialog).getByRole("checkbox", { name: "Restrictors helper" })).toBeDisabled();
    expect(within(dialog).getByRole("checkbox", { name: "Fade the rest" })).toBeDisabled();

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Helpers" }));

    expect(rendered.view.helpers.isShown).toBe(true);

    const helper: HTMLElement = within(dialog).getByRole("checkbox", { name: "Restrictors helper" });

    expect(helper).toBeChecked();
    await userEvent.click(helper);
    expect(rendered.view.helpers.kinds).toEqual({ [ELevelObjectKind.RESTRICTORS]: false });

    act(() => rendered.view.toggleHelpers());

    expect(within(dialog).getByRole("checkbox", { name: "Helpers" })).not.toBeChecked();
  });
});
