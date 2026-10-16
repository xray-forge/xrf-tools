import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { DEFAULT_LEVEL_CAMERA_OPTIONS } from "@/core/level/lib/camera/level-camera-options";
import { LevelViewService } from "@/core/level/services";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderAction(): RenderResult & { view: LevelViewService } {
  window.localStorage.clear();

  const { container, view }: ILevelViewContainerDescriptor = mockLevelViewContainer();

  return { ...renderWithProviders(<LevelCameraAction />, { container }), view };
}

describe("LevelCameraAction", () => {
  it("says what the camera is set to without being opened", () => {
    expect(renderAction().getByTitle(/Camera: 68°, 12 m\/s/)).toBeInTheDocument();
  });

  it("offers what a person flying a level changes", async () => {
    const rendered = renderAction();

    await userEvent.click(rendered.getByRole("button", { name: "Camera" }));

    for (const label of ["Field of view", "Speed", "Boost", "Sensitivity"]) {
      expect(rendered.getByRole("slider", { name: label })).toBeInTheDocument();
    }
  });

  it("changes the whole camera in one place", async () => {
    const rendered = renderAction();

    await userEvent.click(rendered.getByRole("button", { name: "Camera" }));
    fireEvent.change(rendered.getByRole("slider", { name: "Field of view" }), { target: { value: "90" } });

    expect(rendered.view.camera).toEqual({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, fieldOfView: 90 });
  });

  it("puts the camera back", async () => {
    const rendered = renderAction();

    rendered.view.setCamera({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, speed: 40 });

    await userEvent.click(rendered.getByRole("button", { name: "Camera" }));
    await userEvent.click(rendered.getByRole("button", { name: "Back to the default camera" }));

    expect(rendered.view.camera).toEqual(DEFAULT_LEVEL_CAMERA_OPTIONS);
  });
});
