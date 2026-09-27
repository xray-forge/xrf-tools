import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelGoToAction } from "@/core/level/components/preview/LevelGoToAction";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A camera a quarter turn round and level, somewhere every axis has something to say. */
const CAMERA: ILevelCamera = { heading: Math.PI / 2, pitch: 0, position: { x: -243.75, y: 12.5, z: 87.25 } };

describe("LevelGoToAction", () => {
  it("opens the form at where the camera is as it opens", async () => {
    const view: RenderResult = renderWithProviders(<LevelGoToAction readCamera={() => CAMERA} onGoTo={jest.fn()} />);

    expect(view.queryByTestId("level-go-to-form")).toBeNull();

    await userEvent.click(view.getByRole("button", { name: "Go to" }));

    expect(view.getByTestId("level-go-to-form")).toBeInTheDocument();
    expect(view.getByRole("spinbutton", { name: "X" })).toHaveValue(-243.8);
  });

  it("stays shut while disabled", () => {
    const view: RenderResult = renderWithProviders(
      <LevelGoToAction isDisabled readCamera={() => CAMERA} onGoTo={jest.fn()} />
    );

    expect(view.getByRole("button", { name: "Go to" })).toBeDisabled();
  });
});
