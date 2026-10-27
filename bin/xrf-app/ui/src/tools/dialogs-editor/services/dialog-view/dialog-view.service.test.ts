import { beforeEach, describe, expect, it } from "@jest/globals";
import { EDialogsCommand } from "@xrf/ipc/commands/dialogs";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";

import { createRoots } from "@/core/assets/lib/roots";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred, flush, IDeferred } from "@/fixtures/utils/promise";
import { MOCK_DIALOG, MOCK_DIALOGS_PROJECT } from "@/tools/dialogs-editor/fixtures/dialogs.mocks";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogViewService } from "./dialog-view.service";

/** A view over a project already open, as the editor shows one. */
async function mockOpenedView() {
  const { container, service } = mockInjectedService(DialogViewService, [DialogsService]);
  const dialogsService: DialogsService = container.get(DialogsService);

  await dialogsService.openProject(createRoots(["C:\\game"]), "gamedata");

  return { dialogsService, service };
}

describe("DialogViewService", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EDialogsCommand.OPEN_PROJECT]: mockSessionResponse(() => MOCK_DIALOGS_PROJECT),
      [EDialogsCommand.GET_DIALOG]: () => MOCK_DIALOG,
    });
  });

  it("fetches one dialog on selection rather than carrying every dialog in the project", async () => {
    const { service } = await mockOpenedView();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    expect(mockInvoke).toHaveBeenCalledWith(EDialogsCommand.GET_DIALOG, {
      request: {
        sessionId: expect.any(String),
        logicalPath: "configs\\gameplay\\dialogs.xml",
        id: "trader",
        language: null,
      },
    });
    expect(service.dialog.value).toBe(MOCK_DIALOG);
    // Echoed back by the backend, so the bar shows what was actually resolved rather than what was asked.
    expect(service.resolvedLanguage).toBe("eng");
  });

  it("re-fetches the open dialog when the language changes", async () => {
    const { service } = await mockOpenedView();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    service.setLanguage("rus");

    // The index is resident on the backend, so switching costs a lookup rather than a re-read, which
    // is why this re-asks instead of the response carrying every language.
    expect(mockInvoke).toHaveBeenLastCalledWith(EDialogsCommand.GET_DIALOG, {
      request: {
        sessionId: expect.any(String),
        logicalPath: "configs\\gameplay\\dialogs.xml",
        id: "trader",
        language: "rus",
      },
    });
  });

  it("keeps the dialog shown and inspected while its text in another language is fetched", async () => {
    const { service } = await mockOpenedView();
    const answer: IDeferred<typeof MOCK_DIALOG> = deferred();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");
    service.inspectNode("0");

    mockIpcCommands({ [EDialogsCommand.GET_DIALOG]: () => answer.promise });
    service.setLanguage("rus");
    await flush();

    expect(service.dialog.isLoading).toBe(true);
    expect(service.dialog.value).toBe(MOCK_DIALOG);
    expect(service.inspectedNodeId).toBe("0");

    answer.resolve({ ...MOCK_DIALOG, language: "rus" });
    await flush();

    expect(service.dialog.value?.language).toBe("rus");
    expect(service.inspectedNodeId).toBe("0");
  });

  it("shows nothing of the previous dialog while another one is fetched", async () => {
    const { service } = await mockOpenedView();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");
    service.inspectNode("0");

    mockIpcCommands({ [EDialogsCommand.GET_DIALOG]: () => deferred<typeof MOCK_DIALOG>().promise });
    void service.selectDialog("configs\\gameplay\\dialogs.xml", "barman");
    await flush();

    expect(service.dialog.isLoading).toBe(true);
    expect(service.dialog.value).toBeNull();
    expect(service.inspectedNodeId).toBeNull();
  });

  it("keeps the selection when a dialog cannot be read, so the tree still shows what failed", async () => {
    const { service } = await mockOpenedView();

    mockIpcCommands({
      [EDialogsCommand.GET_DIALOG]: () => {
        throw new Error("No dialog 'trader' in 'configs\\gameplay\\dialogs.xml'");
      },
    });

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    expect(service.selection).toEqual({ id: "trader", logicalPath: "configs\\gameplay\\dialogs.xml" });
    expect(service.dialog.value).toBeNull();
    expect(service.dialog.isLoading).toBe(false);
    expect(service.dialog.error).not.toBeNull();
  });

  it("drops the open dialog when a new project is opened", async () => {
    const { dialogsService, service } = await mockOpenedView();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");
    service.setLanguage("rus");
    await dialogsService.openProject(createRoots(["C:\\other"]), "gamedata");

    // A dialog from the previous project would otherwise stay on screen under the new project's tree.
    expect(service.selection).toBeNull();
    expect(service.dialog.value).toBeNull();
    expect(service.language).toBeNull();
  });

  it("resolves in nothing for a project whose text tree was not read", async () => {
    mockIpcCommands({
      [EDialogsCommand.OPEN_PROJECT]: mockSessionResponse(() => ({
        ...MOCK_DIALOGS_PROJECT,
        languages: [],
        textKeys: 0,
      })),
    });

    const { service } = await mockOpenedView();

    // Nothing to resolve in, which is what stops the bar offering a language the project cannot show.
    expect(service.resolvedLanguage).toBeNull();
  });

  it("places the open dialog in the listing, and nothing once it is cleared", async () => {
    const { service } = await mockOpenedView();

    // Nothing is open yet, so there is no row to place.
    expect(service.selectedPath).toBeNull();

    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    expect(service.selectedPath).toBe("dialogs.xml\\trader");

    service.clearSelection();

    expect(service.selectedPath).toBeNull();
  });

  // The pair addresses a dialog; a project that no longer holds it can only answer that it is not there.
  it("places no row for a selection the open project does not hold", async () => {
    const { service } = await mockOpenedView();

    await service.selectDialog("configs\\gameplay\\absent.xml", "trader");

    expect(service.selectedPath).toBeNull();
  });
});
