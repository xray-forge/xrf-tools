import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelPreviewGoTo } from "@/core/level/components/preview/LevelPreviewGoTo";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { LevelRenderCamera } from "@/core/level/services/level-render/level-render.service.camera";

/** A camera a quarter turn round and level, somewhere every axis has something to say. */
const CAMERA: ILevelCamera = { heading: Math.PI / 2, pitch: 0, position: { x: -243.75, y: 12.5, z: 87.25 } };

/**
 * @returns The field, opened over a camera standing at `CAMERA`, and what it asks the camera to go to.
 */
function renderField(): { rendered: TLevelViewRenderResult; goTo: jest.SpiedFunction<(goTo: ILevelGoTo) => void> } {
  // Spied before the container builds the camera, whose bound action is fixed on it as it is made.
  const goTo = jest.spyOn(LevelRenderCamera.prototype, "goTo").mockImplementation(() => {});
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.viewport.noteCamera(CAMERA);
  descriptor.viewport.openGoTo();

  return { goTo, rendered: renderLevelView(<LevelPreviewGoTo />, descriptor) };
}

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelPreviewGoTo", () => {
  it("starts as the camera readout reads, the whole of it selected to type or paste over", () => {
    const { rendered } = renderField();
    const field: HTMLInputElement = rendered.getByRole("textbox", { name: "Coordinates" }) as HTMLInputElement;

    expect(field).toHaveValue("x -243.8 y 12.5 z 87.3 h 90.0° p 0.0°");
    expect(field).toHaveFocus();
    expect([field.selectionStart, field.selectionEnd]).toEqual([0, field.value.length]);
  });

  it("goes where pasted text says on Enter, what it leaves out kept, and closes", async () => {
    const { rendered, goTo } = renderField();
    const field: HTMLElement = rendered.getByRole("textbox", { name: "Coordinates" });

    await userEvent.clear(field);
    await userEvent.paste("x -394.9 y 4.6 z 167.7");

    expect(rendered.getByText(/^Enter goes to x -394\.9 y 4\.6 z 167\.7 h 90\.0° p 0\.0°$/)).toBeInTheDocument();

    await userEvent.keyboard("{Enter}");

    expect(goTo).toHaveBeenCalledWith({ heading: 90, pitch: 0, x: -394.9, y: 4.6, z: 167.7 });
    expect(rendered.viewport.isGoingTo).toBe(false);
  });

  it("goes nowhere from text naming no place, and says what it takes", async () => {
    const { rendered, goTo } = renderField();
    const field: HTMLElement = rendered.getByRole("textbox", { name: "Coordinates" });

    await userEvent.clear(field);
    await userEvent.type(field, "somewhere{Enter}");

    expect(rendered.getByText(/^Names no place/)).toBeInTheDocument();
    expect(goTo).not.toHaveBeenCalled();
    expect(rendered.viewport.isGoingTo).toBe(true);
  });

  it("leaves on Escape without going anywhere, the selection kept", async () => {
    const { rendered, goTo } = renderField();

    await userEvent.keyboard("{Escape}");

    expect(goTo).not.toHaveBeenCalled();
    expect(rendered.viewport.isGoingTo).toBe(false);
  });
});
