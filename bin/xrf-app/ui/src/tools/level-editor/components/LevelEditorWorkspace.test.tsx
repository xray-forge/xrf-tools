import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { resetMockInvoke } from "@xrf/ipc/testing";

import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelEditorWorkspace } from "@/tools/level-editor/components/LevelEditorWorkspace";
import { container as LEVEL_EDITOR_CONTAINER } from "@/tools/level-editor/runtime";

const SCENE: string = "C:\\rawdata\\levels\\labx8.level";

function renderLayout(
  overrides: Partial<Parameters<typeof LevelEditorWorkspace>[0]> = {},
  prepare: (container: Container) => void = () => undefined
): RenderResult {
  const container: Container = mockContainer([...(LEVEL_EDITOR_CONTAINER.bindings ?? [])]);

  prepare(container);

  return renderWithProviders(
    <LevelEditorWorkspace name={SCENE} renderViewport={() => <div data-testid={"stub-viewport"} />} {...overrides} />,
    { container, route: "/level-editor" }
  );
}

describe("LevelEditorWorkspace", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("heads the workspace with the scene and covers its viewport until the scene is drawn", () => {
    const { getByTestId } = renderLayout();

    expect(getByTestId("level-editor-file-header")).toHaveTextContent(SCENE);
    expect(getByTestId("stub-viewport")).toBeInTheDocument();
    expect(getByTestId("level-preview-cover")).toHaveClass("opacity-100");
    expect(getByTestId("level-preview-cover")).toHaveTextContent("Preparing the scene…");
  });

  it("says what the scene's load waits for", () => {
    const { getByTestId } = renderLayout({}, (container: Container) =>
      container
        .get(LevelViewportService)
        .noteLoad(mockRenderLoadReport({ isReady: false, textures: 3, texturesTotal: 40 }))
    );

    expect(getByTestId("level-preview-cover")).toHaveTextContent("Uploading textures, 3 of 40…");
  });

  it("says it compiles the scene's objects once its textures are in", () => {
    const { getByTestId } = renderLayout({}, (container: Container) =>
      container.get(LevelViewportService).noteLoad(mockRenderLoadReport({ isReady: false }))
    );

    expect(getByTestId("level-preview-cover")).toHaveTextContent("Compiling the scene's objects…");
  });

  it("heads the workspace with the scene being opened, and says it is opening", () => {
    const { getByTestId } = renderLayout({ isLoading: true, name: null, pending: SCENE });

    expect(getByTestId("level-editor-file-header")).toHaveTextContent(SCENE);
    expect(getByTestId("level-preview-cover")).toHaveTextContent("Opening scene…");
  });

  it("says a scene changed since it was read", () => {
    const { getByTestId } = renderLayout({ isStale: true });

    expect(getByTestId("level-editor-file-header")).toHaveTextContent("Changed since read");
  });

  it("reports why an open failed and offers it again", () => {
    const onRetry = jest.fn();
    const { getByRole, queryByTestId } = renderLayout({ error: "The .level holds no [version]", name: null, onRetry });

    expect(getByRole("alert")).toHaveTextContent("The .level holds no [version]");
    expect(queryByTestId("level-preview-cover")).toBeNull();

    fireEvent.click(getByRole("button", { name: "Retry" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("returns to the open form through the tool breadcrumb, and closes the scene from its header", () => {
    const onBack = jest.fn();
    const onClose = jest.fn();
    const { getByRole } = renderLayout({ onBack, onClose });

    fireEvent.click(getByRole("button", { name: "Back to Level editor" }));
    fireEvent.click(getByRole("button", { name: "Close scene" }));

    expect(onBack).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("offers the camera's lens and speeds in the toolbar", () => {
    const { getByTestId } = renderLayout();

    expect(getByTestId("level-camera-action")).toBeInTheDocument();
  });

  it("publishes the Outliner on the left, which lists nothing while no scene is open", () => {
    const { getByText } = renderLayout({ name: null });

    expect(getByText("No scene open. Open one to list what it holds.")).toBeInTheDocument();
  });
});
