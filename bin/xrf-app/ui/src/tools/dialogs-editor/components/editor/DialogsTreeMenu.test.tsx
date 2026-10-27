import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { EDialogsCommand } from "@xrf/ipc/commands/dialogs";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { DialogProjectDescriptor } from "@xrf/ipc/types/xrf-dialog";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogsTreeMenu } from "./DialogsTreeMenu";

const PROJECT: DialogProjectDescriptor = {
  mode: "gamedata",
  roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
  dialogsPrefix: "configs\\gameplay",
  translationsPrefix: "configs\\text",
  isEditable: true,
  languages: ["eng"],
  textKeys: 1,
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

async function renderMenu(): Promise<{ render: RenderResult; container: Container }> {
  // A dialog read answers the null every unlisted command does: these tests count reads, never render one.
  mockIpcCommands({ [EDialogsCommand.RESTORE_PROJECT]: mockSessionResponse(PROJECT) });

  const { container, service } = mockInjectedService(DialogsService, [DialogViewService]);
  const render = renderWithProviders(<DialogsTreeMenu />, { container });

  await waitFor(() => expect(service.isReady).toBe(true));

  return { container, render };
}

describe("DialogsTreeMenu", () => {
  it("reads a dialog on a double click, never on a single one", async () => {
    const { render } = await renderMenu();

    fireEvent.dblClick(render.getByText("dialogs.xml"));

    const leaf: HTMLElement = await render.findByText("trader");

    fireEvent.click(leaf);

    expect(mockInvoke).not.toHaveBeenCalledWith(EDialogsCommand.GET_DIALOG, expect.anything());
    expect(leaf.closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");

    fireEvent.dblClick(leaf);

    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith(EDialogsCommand.GET_DIALOG, expect.anything()));
  });
});
