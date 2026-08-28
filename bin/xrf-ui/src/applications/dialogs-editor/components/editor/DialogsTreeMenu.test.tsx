import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { DialogsTreeMenu } from "@/applications/dialogs-editor/components/editor/DialogsTreeMenu";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { createLoadable } from "@/lib/loadable";

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

function renderMenu(): { render: RenderResult; container: Container } {
  setMockInvokeResponses({ ["plugin:dialogs|get_dialog"]: null });

  const { container, service } = mockInjectedService(DialogsService);

  service.project = createLoadable(PROJECT);

  return { container, render: renderWithProviders(<DialogsTreeMenu />, { container }) };
}

describe("DialogsTreeMenu", () => {
  it("reads a dialog on a double click, never on a single one", async () => {
    const { render } = renderMenu();

    fireEvent.dblClick(render.getByText("dialogs.xml"));

    const leaf: HTMLElement = await render.findByText("trader");

    fireEvent.click(leaf);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:dialogs|get_dialog", expect.anything());
    expect(leaf.closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");

    fireEvent.dblClick(leaf);

    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("plugin:dialogs|get_dialog", expect.anything()));
  });
});
