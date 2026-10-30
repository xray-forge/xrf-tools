import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent, within } from "@testing-library/react";

import { LevelCameraEffectsAction } from "@/core/level/components/preview/toolbar/camera/LevelCameraEffectsAction";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { DEFAULT_RENDER_CAMERA_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelCameraEffectsAction", () => {
  it("turns the depth of field on for the view alone and arms a click in the view to focus it", () => {
    const { container, renderSettings } = mockLevelViewContainer();

    renderSettings.setRendererOverrides({ camera: DEFAULT_RENDER_CAMERA_SETTINGS });

    const { getByRole, getByTestId } = renderWithProviders(<LevelCameraEffectsAction />, { container });

    expect(getByRole("button", { name: "Camera effects" })).toBeInTheDocument();

    fireEvent.click(getByRole("button", { name: "Camera effects" }));

    const section: HTMLElement = getByTestId("level-camera-depth-of-field");

    expect(within(section).getByRole("slider", { name: "Aperture" })).toBeDisabled();

    act(() => fireEvent.click(within(section).getByRole("checkbox", { name: "Depth of field" })));

    expect(container.get(LevelViewService).features.camera).toEqual({ isDefocused: true });
    expect(within(section).getByRole("slider", { name: "Aperture" })).toHaveAttribute("aria-valuetext", "f/2.8");

    act(() => fireEvent.click(getByTestId("level-camera-focus-pick")));

    expect(container.get(LevelViewportService).isFocusPicking).toBe(true);
    expect(getByTestId("level-camera-focus-pick")).toHaveTextContent("Click in the view to focus");
  });
});
