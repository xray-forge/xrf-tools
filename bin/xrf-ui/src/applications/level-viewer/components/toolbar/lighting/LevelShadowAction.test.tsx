import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelShadowAction } from "@/applications/level-viewer/components/toolbar/lighting/LevelShadowAction";
import { ERenderContactShadowMode } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_RENDER_SHADOW_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param shadows - The shadows the settings draw every viewport with.
 * @returns The rendered action and the services behind it.
 */
function renderAction(shadows: TRenderShadowSettings = DEFAULT_RENDER_SHADOW_SETTINGS): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ shadows });

  return renderLevelView(<LevelShadowAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelShadowAction", () => {
  it("sets the view's own cascades over the settings'", async () => {
    const { getByRole, findByRole, view } = renderAction();

    expect(getByRole("button", { name: "Sun shadows" })).toHaveAccessibleDescription(
      "Sun shadows in 4 cascades, the widest 480 m across, at 2048. Right-click for its settings"
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Sun shadows" }) });
    await findByRole("dialog", { name: "Sun shadows" });
    await userEvent.click(getByRole("button", { name: "1" }));

    expect(view.features.shadows).toEqual({ cascades: [20] });
  });

  it("turns the view's contact shadows on, offering their strengths only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = renderAction();

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Sun shadows" }) });
    await findByRole("dialog", { name: "Sun shadows" });

    expect(queryByRole("slider", { name: "Contact length" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    // Only the mode is the view's own, so the strengths still follow the settings.
    expect(view.features.shadows).toEqual({ contact: { mode: ERenderContactShadowMode.ENHANCED } });
    expect(getByRole("slider", { name: "Contact length" })).toBeInTheDocument();
    expect(getByRole("slider", { name: "Contact lights" })).toHaveAttribute("aria-valuetext", "4 lights");
    expect(getByRole("button", { hidden: true, name: "Sun shadows" })).toHaveAccessibleDescription(
      "Sun shadows in 4 cascades, the widest 480 m across, at 2048, with contact shadows. Right-click for its settings"
    );
  });

  it("opens on a click, saying why, while the settings draw no shadows", async () => {
    const { getByRole, findByRole } = renderAction({ ...DEFAULT_RENDER_SHADOW_SETTINGS, isEnabled: false });

    expect(getByRole("button", { name: "Sun shadows" })).toHaveAccessibleDescription(
      "Sun shadows are off in Settings, under Rendering"
    );

    await userEvent.click(getByRole("button", { name: "Sun shadows" }));

    expect(await findByRole("dialog", { name: "Sun shadows" })).toHaveTextContent(
      "Sun shadows are off in Settings, under Rendering"
    );
  });
});
