import { beforeEach, describe, expect, it } from "@jest/globals";
import { EDialogsCommand } from "@xrf/ipc/commands/dialogs";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";

import { MOCK_DIALOGS_PROJECT } from "@/applications/dialogs-editor/fixtures/dialogs.mocks";
import { createRoots } from "@/core/assets/lib/roots";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { DialogsService } from "./dialogs.service";

describe("DialogsService", () => {
  beforeEach(() => {
    mockIpcCommands({ [EDialogsCommand.RESTORE_PROJECT]: mockSessionResponse(() => null) });
  });

  it("opens a project over roots and the layout mode", async () => {
    const { service } = mockInjectedService(DialogsService);

    mockIpcCommands({ [EDialogsCommand.OPEN_PROJECT]: mockSessionResponse(() => MOCK_DIALOGS_PROJECT) });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");

    // One request object rather than loose arguments, which is how every command with more than three is shaped.
    expect(mockInvoke).toHaveBeenCalledWith(EDialogsCommand.OPEN_PROJECT, {
      request: {
        sessionId: expect.any(String),
        roots: createRoots(["C:\\game"]),
        mode: "gamedata",
        dialogsPrefix: null,
        translationsPrefix: null,
      },
    });
    expect(service.project.value).toEqual(MOCK_DIALOGS_PROJECT);
    expect(service.languages).toEqual(["eng", "rus"]);
  });

  it("reports no languages for a project whose text tree was not read", async () => {
    const { service } = mockInjectedService(DialogsService);

    mockIpcCommands({
      [EDialogsCommand.OPEN_PROJECT]: mockSessionResponse(() => ({
        ...MOCK_DIALOGS_PROJECT,
        languages: [],
        textKeys: 0,
      })),
    });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");

    expect(service.languages).toEqual([]);
  });

  it("lists every dialog of the project, which is how a tree addresses one", async () => {
    const { service } = mockInjectedService(DialogsService);

    mockIpcCommands({ [EDialogsCommand.OPEN_PROJECT]: mockSessionResponse(() => MOCK_DIALOGS_PROJECT) });

    await service.openProject(createRoots(["C:\\game"]), "gamedata");

    expect(service.entries).toEqual([
      {
        path: "dialogs.xml\\trader",
        payload: { logicalPath: "configs\\gameplay\\dialogs.xml", id: "trader" },
      },
    ]);
  });
});
