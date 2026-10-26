import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelCameraAction } from "@/core/level/components/preview/toolbar/camera/LevelCameraAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_CAMERA_OPTIONS } from "@/core/level/lib/camera/level-camera-options";
import { clearLocalStorage } from "@/core/storage";

/**
 * @param setup - What the case sets on the services before it draws.
 * @returns The camera's action and the services behind it.
 */
function renderAction(setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  setup(descriptor);

  return renderLevelView(<LevelCameraAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

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

    expect(rendered.view.cameraOptions).toEqual({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, fieldOfView: 90 });
  });

  it("puts the camera back", async () => {
    const rendered = renderAction(({ view }) => view.setCameraOptions({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, speed: 40 }));

    await userEvent.click(rendered.getByRole("button", { name: "Camera" }));
    await userEvent.click(rendered.getByRole("button", { name: "Reset the camera" }));

    expect(rendered.view.cameraOptions).toEqual(DEFAULT_LEVEL_CAMERA_OPTIONS);
  });
});
