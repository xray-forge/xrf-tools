import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { resetMockInvoke } from "@xrf/ipc/testing";

import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelEditorLayout } from "@/tools/level-editor/components/LevelEditorLayout";
import { container as LEVEL_EDITOR_CONTAINER } from "@/tools/level-editor/runtime";

const SCENE: string = "C:\\rawdata\\levels\\labx8.level";

function renderLayout(overrides: Partial<Parameters<typeof LevelEditorLayout>[0]> = {}): RenderResult {
  const container: Container = mockContainer([...(LEVEL_EDITOR_CONTAINER.bindings ?? [])]);

  return renderWithProviders(<LevelEditorLayout name={SCENE} {...overrides} />, {
    container,
    route: "/level-editor",
  });
}

describe("LevelEditorLayout", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("heads the workspace with the scene and says it is not drawn yet", () => {
    const { getByTestId, getByText } = renderLayout();

    expect(getByTestId("level-editor-file-header")).toHaveTextContent(SCENE);
    expect(getByText("The scene is not drawn yet")).toBeInTheDocument();
  });

  it("heads the workspace with the scene being opened, and says it is opening", () => {
    const { getByRole, getByTestId } = renderLayout({ isLoading: true, name: null, pending: SCENE });

    expect(getByTestId("level-editor-file-header")).toHaveTextContent(SCENE);
    expect(getByRole("status")).toHaveTextContent("Opening scene");
  });

  it("says a scene changed since it was read", () => {
    const { getByTestId } = renderLayout({ isStale: true });

    expect(getByTestId("level-editor-file-header")).toHaveTextContent("Changed since read");
  });

  it("reports why an open failed and offers it again", () => {
    const onRetry = jest.fn();
    const { getByRole } = renderLayout({ error: "The .level holds no [version]", name: null, onRetry });

    expect(getByRole("alert")).toHaveTextContent("The .level holds no [version]");

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

  it("publishes the Outliner on the left, which lists nothing while no scene is open", () => {
    const { getByText } = renderLayout({ name: null });

    expect(getByText("No scene open. Open one to list what it holds.")).toBeInTheDocument();
  });
});
