import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { DialogsEditorApplication } from "@/applications/dialogs-editor/DialogsEditorApplication";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor, DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
import { ProjectService } from "@/core/settings/services/project";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const PROJECT: DialogProjectDescriptor = {
  mode: "gamedata",
  roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
  dialogsPrefix: "configs\\gameplay",
  translationsPrefix: "configs\\text",
  isEditable: true,
  languages: ["eng", "rus"],
  textKeys: 12,
  files: {
    "configs\\gameplay\\dialogs.xml": {
      physicalPath: "C:/game/configs/gameplay/dialogs.xml",
      isEditable: true,
      encoding: "windows-1251",
      dialogs: [{ id: "trader", phrases: 1, priority: null }],
    },
  },
  findings: [],
};

const DIALOG: DialogDescriptor = {
  logicalPath: "configs\\gameplay\\dialogs.xml",
  id: "trader",
  priority: null,
  elements: [],
  language: "eng",
  phrases: [
    {
      id: "0",
      textKey: "trader_hello",
      text: "Hello, stalker",
      isFinal: false,
      isInPhraseList: true,
      next: [],
      elements: [],
    },
  ],
};

function renderEditor(): RenderResult {
  return renderWithProviders(<DialogsEditorApplication />, {
    bindings: [ProjectService, DialogsService],
    route: "/dialogs-editor",
  });
}

describe("opened dialogs editor", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:dialogs|get_project"]: PROJECT,
      ["plugin:dialogs|get_dialog"]: DIALOG,
      ["plugin:dialogs|close_project"]: undefined,
    });
  });

  it("invites a selection before anything is picked", async () => {
    const { findByText } = renderEditor();

    expect(await findByText("No dialog selected")).toBeInTheDocument();
  });

  it("draws the graph for the dialog that was picked", async () => {
    const { findByText, findByTestId } = renderEditor();

    await userEvent.click(await findByText("trader"));

    expect(await findByTestId("dialog-graph")).toBeInTheDocument();
  });

  it("says why a dialog could not be read instead of reading forever", async () => {
    // The defect this covers: the placeholder branched on the selection rather than on the resource,
    // so a failed read reported "Reading dialog" permanently and the error never reached the surface.
    setMockInvokeResponses({
      ["plugin:dialogs|get_project"]: PROJECT,
      ["plugin:dialogs|get_dialog"]: () => {
        throw new Error("No dialog 'trader' in 'configs\\gameplay\\dialogs.xml'");
      },
    });

    const { findByText, queryByText } = renderEditor();

    await userEvent.click(await findByText("trader"));

    expect(await findByText("Could not read this dialog")).toBeInTheDocument();
    expect(queryByText("Reading dialog")).not.toBeInTheDocument();
  });

  it("offers the languages the project read and nothing when it read none", async () => {
    const { findByLabelText } = renderEditor();

    expect(await findByLabelText("Language")).toBeInTheDocument();
  });

  it("hides the language selector for a project with no text tree", async () => {
    // A switcher over no languages offers a choice that cannot be made.
    setMockInvokeResponses({
      ["plugin:dialogs|get_project"]: { ...PROJECT, languages: [], textKeys: 0 },
      ["plugin:dialogs|get_dialog"]: DIALOG,
    });

    const { findByText, queryByLabelText } = renderEditor();

    await findByText("No dialog selected");

    expect(queryByLabelText("Language")).not.toBeInTheDocument();
  });
});
