import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderAction(onChange: (camera: ILevelCameraOptions) => void = jest.fn()): RenderResult {
  return renderWithProviders(<LevelCameraAction camera={DEFAULT_LEVEL_CAMERA_OPTIONS} onChange={onChange} />);
}

describe("LevelCameraAction", () => {
  it("says what the camera is set to without being opened", () => {
    expect(renderAction().getByTitle(/Camera: 68°, 12 m\/s/)).toBeInTheDocument();
  });

  it("offers what a person flying a level changes", async () => {
    const view: RenderResult = renderAction();

    await userEvent.click(view.getByRole("button", { name: "Camera" }));

    for (const label of ["Field of view", "Speed", "Boost", "Sensitivity"]) {
      expect(view.getByRole("slider", { name: label })).toBeInTheDocument();
    }
  });

  it("answers with the whole camera, changed in one place", async () => {
    const onChange = jest.fn<(camera: ILevelCameraOptions) => void>();
    const view: RenderResult = renderAction(onChange);

    await userEvent.click(view.getByRole("button", { name: "Camera" }));
    fireEvent.change(view.getByRole("slider", { name: "Field of view" }), { target: { value: "90" } });

    expect(onChange).toHaveBeenCalledWith({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, fieldOfView: 90 });
  });

  it("puts the camera back", async () => {
    const onChange = jest.fn<(camera: ILevelCameraOptions) => void>();
    const view: RenderResult = renderAction(onChange);

    await userEvent.click(view.getByRole("button", { name: "Camera" }));
    await userEvent.click(view.getByRole("button", { name: "Back to the default camera" }));

    expect(onChange).toHaveBeenCalledWith(DEFAULT_LEVEL_CAMERA_OPTIONS);
  });
});
