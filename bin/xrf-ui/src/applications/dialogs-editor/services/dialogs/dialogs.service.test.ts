import { beforeEach, describe, expect, it } from "@jest/globals";

import { DialogsService } from "@/applications/dialogs-editor/services/dialogs/dialogs.service";
import { createRoots } from "@/core/assets/lib/roots";
import { DialogDescriptor, DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const PROJECT: DialogProjectDescriptor = {
  mode: "gamedata",
  roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
  dialogsPrefix: "configs\\gameplay",
  translationsPrefix: "configs\\text",
  isEditable: true,
  languages: ["eng", "rus"],
  textKeys: 24802,
  files: {
    "configs\\gameplay\\dialogs.xml": {
      physicalPath: "C:/game/configs/gameplay/dialogs.xml",
      isEditable: true,
      encoding: "windows-1251",
      dialogs: [{ id: "trader", phrases: 3, priority: null }],
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
      next: ["1"],
      elements: [],
    },
  ],
};

describe("DialogsService", () => {
  beforeEach(() => {
    setMockInvokeResponses({ ["plugin:dialogs|get_project"]: () => null });
  });

  it("opens a project over roots and the layout mode", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({ ["plugin:dialogs|open_project"]: () => PROJECT });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");

    expect(mockInvoke).toHaveBeenCalledWith("plugin:dialogs|open_project", {
      roots: createRoots(["C:\\game"]),
      mode: "gamedata",
      dialogsPrefix: null,
      translationsPrefix: null,
    });
    expect(service.project.value).toBe(PROJECT);
    expect(service.languages).toEqual(["eng", "rus"]);
  });

  it("fetches one dialog on selection rather than carrying every dialog in the project", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({
      ["plugin:dialogs|open_project"]: () => PROJECT,
      ["plugin:dialogs|get_dialog"]: () => DIALOG,
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");
    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    expect(mockInvoke).toHaveBeenCalledWith("plugin:dialogs|get_dialog", {
      logicalPath: "configs\\gameplay\\dialogs.xml",
      id: "trader",
      language: null,
    });
    expect(service.dialog.value).toBe(DIALOG);
    // Echoed back by the backend, so the bar shows what was actually resolved rather than what was asked.
    expect(service.resolvedLanguage).toBe("eng");
  });

  it("re-fetches the open dialog when the language changes", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({
      ["plugin:dialogs|open_project"]: () => PROJECT,
      ["plugin:dialogs|get_dialog"]: () => DIALOG,
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");
    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    service.setLanguage("rus");

    // The index is resident on the backend, so switching costs a lookup rather than a re-read, which
    // is why this re-asks instead of the response carrying every language.
    expect(mockInvoke).toHaveBeenLastCalledWith("plugin:dialogs|get_dialog", {
      logicalPath: "configs\\gameplay\\dialogs.xml",
      id: "trader",
      language: "rus",
    });
  });

  it("keeps the selection when a dialog cannot be read, so the tree still shows what failed", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({
      ["plugin:dialogs|open_project"]: () => PROJECT,
      ["plugin:dialogs|get_dialog"]: () => {
        throw new Error("No dialog 'trader' in 'configs\\gameplay\\dialogs.xml'");
      },
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");
    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");

    expect(service.selection).toEqual({ id: "trader", logicalPath: "configs\\gameplay\\dialogs.xml" });
    expect(service.dialog.value).toBeNull();
    expect(service.dialog.isLoading).toBe(false);
    expect(service.dialog.error).not.toBeNull();
  });

  it("drops the open dialog when a new project is opened", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({
      ["plugin:dialogs|open_project"]: () => PROJECT,
      ["plugin:dialogs|get_dialog"]: () => DIALOG,
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");
    await service.selectDialog("configs\\gameplay\\dialogs.xml", "trader");
    await service.openProject(createRoots(["C:\\other"]), "gamedata");

    // A dialog from the previous project would otherwise stay on screen under the new project's tree.
    expect(service.selection).toBeNull();
    expect(service.dialog.value).toBeNull();
  });

  it("reports no languages for a project whose text tree was not read", async () => {
    const { service } = mockInjectedService(DialogsService);

    setMockInvokeResponses({
      ["plugin:dialogs|open_project"]: () => ({ ...PROJECT, languages: [], textKeys: 0 }),
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");

    expect(service.languages).toEqual([]);
    // Nothing to resolve in, which is what stops the bar offering a language the project cannot show.
    expect(service.resolvedLanguage).toBeNull();
  });
});
