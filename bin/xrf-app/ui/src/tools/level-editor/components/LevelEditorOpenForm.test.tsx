import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { listMockInvokeCalls, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";

import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockScenePartResponses, mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorOpenForm } from "./LevelEditorOpenForm";

const SCENE: string = "C:\\rawdata\\levels\\labx8.level";

function renderForm(onFinished: () => void = () => undefined): RenderResult {
  const container: Container = mockContainer([LevelEditorSceneService]);

  return renderWithProviders(<LevelEditorOpenForm onFinished={onFinished} />, { container, route: "/level-editor" });
}

function setScene(form: RenderResult, path: string): void {
  fireEvent.change(form.getByRole("textbox", { name: "Scene" }), { target: { value: path } });
}

describe("LevelEditorOpenForm", () => {
  beforeEach(() => {
    clearLocalStorage();
    resetMockInvoke();

    mockIpcCommands({
      [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
      ...mockScenePartResponses(),
    });
  });

  it("opens nothing until a scene is named", () => {
    const { getByRole } = renderForm();

    expect(getByRole("button", { name: "Open" })).toBeDisabled();
  });

  // No root named is the backend's default, `objects` beside the scene's directory, which the request leaves to it.
  it("opens the scene named, leaving its objects roots to the default", async () => {
    const onFinished = jest.fn();
    const form: RenderResult = renderForm(onFinished);

    setScene(form, SCENE);
    await userEvent.click(form.getByRole("button", { name: "Open" }));

    expect(listMockInvokeCalls(ELevelsCommand.OPEN_SCENE)[0]?.request).toEqual({
      objectsRoots: null,
      source: { path: SCENE },
    });
    expect(onFinished).toHaveBeenCalledTimes(1);
  });

  it("opens the scene with the objects roots named, in order, leaving out empty ones", async () => {
    const form: RenderResult = renderForm();

    setScene(form, SCENE);
    await userEvent.click(form.getByRole("button", { name: "Add objects root" }));
    await userEvent.type(form.getByRole("textbox", { name: "Add objects root 1" }), "C:\\sdk\\objects");
    await userEvent.click(form.getByRole("button", { name: "Add objects root" }));
    await userEvent.click(form.getByRole("button", { name: "Add objects root" }));
    await userEvent.type(form.getByRole("textbox", { name: "Add objects root 3" }), " D:\\mod\\objects ");
    await userEvent.click(form.getByRole("button", { name: "Open" }));

    expect(listMockInvokeCalls(ELevelsCommand.OPEN_SCENE)[0]?.request.objectsRoots).toEqual([
      "C:\\sdk\\objects",
      "D:\\mod\\objects",
    ]);
  });

  it("says why a scene could not be opened", async () => {
    mockIpcCommands({
      [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(() => {
        throw new Error("The .level holds no [version]");
      }),
    });

    const form: RenderResult = renderForm();

    setScene(form, SCENE);
    await userEvent.click(form.getByRole("button", { name: "Open" }));

    expect(await form.findByText("The .level holds no [version]")).toBeInTheDocument();
  });
});
