import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Nullable } from "@xrf/types";

import { LevelGoToForm } from "@/core/level/components/preview/LevelGoToForm";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A camera a quarter turn round and level, somewhere every axis has something to say. */
const CAMERA: ILevelCamera = { heading: Math.PI / 2, pitch: 0, position: { x: -243.75, y: 12.5, z: 87.25 } };

function renderForm(
  onGoTo: (goTo: ILevelGoTo) => void = jest.fn(),
  camera: Nullable<ILevelCamera> = CAMERA
): RenderResult {
  return renderWithProviders(<LevelGoToForm readCamera={() => camera} onGoTo={onGoTo} />);
}

describe("LevelGoToForm", () => {
  it("starts at where the camera is as it mounts, or at the origin before the viewport has drawn", () => {
    const view: RenderResult = renderForm();

    expect(view.getByRole("spinbutton", { name: "Heading °" })).toHaveValue(90);

    view.unmount();

    expect(renderForm(jest.fn(), null).getByRole("spinbutton", { name: "X" })).toHaveValue(0);
  });

  it("goes where a pasted readout says, what it leaves out kept", async () => {
    const onGoTo = jest.fn<(goTo: ILevelGoTo) => void>();
    const view: RenderResult = renderForm(onGoTo);

    fireEvent.change(view.getByLabelText("Paste the readout"), { target: { value: "x -394.9 y 4.6 z 167.7" } });
    await userEvent.click(view.getByRole("button", { name: "Go" }));

    expect(onGoTo).toHaveBeenCalledWith({ heading: 90, pitch: 0, x: -394.9, y: 4.6, z: 167.7 });
  });

  it("goes nowhere while a field is not a number", () => {
    const view: RenderResult = renderForm();

    fireEvent.change(view.getByRole("spinbutton", { name: "Y" }), { target: { value: "" } });

    expect(view.getByRole("spinbutton", { name: "Y" })).toHaveAttribute("aria-invalid", "true");
    expect(view.getByRole("button", { name: "Go" })).toBeDisabled();
  });
});
