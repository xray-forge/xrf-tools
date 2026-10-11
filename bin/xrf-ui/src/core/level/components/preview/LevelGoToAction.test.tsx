import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelGoToAction } from "@/core/level/components/preview/LevelGoToAction";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelViewContainer } from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A camera a quarter turn round and level, somewhere every axis has something to say. */
const CAMERA: ILevelCamera = { heading: Math.PI / 2, pitch: 0, position: { x: -243.75, y: 12.5, z: 87.25 } };

afterEach(() => {
  resetMockInvoke();
});

describe("LevelGoToAction", () => {
  it("opens the form at where the camera is as it opens", async () => {
    setMockInvokeResponses({ ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()) });

    const { container, load, viewport } = mockLevelViewContainer();
    const rendered = renderWithProviders(<LevelGoToAction />, { container });

    // The level the backend holds, restored as the container is provisioned, is what can be gone about in.
    await waitFor(() => expect(load.level.value).not.toBeNull());
    viewport.noteCamera(CAMERA);

    expect(rendered.queryByTestId("level-go-to-form")).toBeNull();

    await userEvent.click(rendered.getByRole("button", { name: "Go to" }));

    expect(rendered.getByTestId("level-go-to-form")).toBeInTheDocument();
    expect(rendered.getByRole("spinbutton", { name: "X" })).toHaveValue(-243.8);
  });

  it("stays shut with no level open", () => {
    const { container } = mockLevelViewContainer();
    const rendered = renderWithProviders(<LevelGoToAction />, { container });

    expect(rendered.getByRole("button", { name: "Go to" })).toBeDisabled();
  });
});
