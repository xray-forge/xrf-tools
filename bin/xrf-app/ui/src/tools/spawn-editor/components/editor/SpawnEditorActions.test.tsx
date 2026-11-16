import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open, save } from "@tauri-apps/plugin-dialog";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockSpawnSession } from "@/tools/spawn-editor/fixtures/spawn.mocks";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorActions } from "./SpawnEditorActions";

const SAVE_PATH: string = "C:\\out\\all.spawn";
const EXPORT_PATH: string = "C:\\out\\unpacked";

interface IActionsRender {
  render: RenderResult;
  service: SpawnEditorService;
}

async function renderActions(): Promise<IActionsRender> {
  mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });

  const { container, service } = mockInjectedService(SpawnEditorService);

  await service.open("C:\\game\\all.spawn");

  const render: RenderResult = await act(async () =>
    renderWithProviders(<SpawnEditorActions />, { container, route: "/spawn-editor" })
  );

  return { render, service };
}

describe("SpawnEditorActions", () => {
  const mockSave = jest.mocked(save);
  const mockOpen = jest.mocked(open<{ directory: true; multiple: false }>);

  beforeEach(() => {
    mockSave.mockResolvedValue(null);
    mockOpen.mockResolvedValue(null);
  });

  it("saves the open spawn where the save dialog says", async () => {
    mockSave.mockResolvedValue(SAVE_PATH);

    const {
      render: { getByRole },
      service,
    }: IActionsRender = await renderActions();

    await userEvent.click(getByRole("button", { name: "Save spawn file" }));

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_FILE)).toEqual([{ path: SAVE_PATH, sessionId: service.sessionId }]);
  });

  it("writes nothing when the save dialog is dismissed", async () => {
    const {
      render: { getByRole },
    }: IActionsRender = await renderActions();

    await userEvent.click(getByRole("button", { name: "Save spawn file" }));

    expect(mockSave).toHaveBeenCalledTimes(1);
    expect(listMockInvokeCalls(ESpawnCommand.SAVE_FILE)).toHaveLength(0);
  });

  it("asks before exporting over a directory, and exports once confirmed", async () => {
    mockOpen.mockResolvedValue(EXPORT_PATH);

    const {
      render: { findByRole, getByRole },
      service,
    }: IActionsRender = await renderActions();

    await userEvent.click(getByRole("button", { name: "Export spawn file" }));

    const dialog = await findByRole("dialog");

    expect(dialog).toHaveTextContent(`Writes one file per chunk into ${EXPORT_PATH}`);
    expect(listMockInvokeCalls(ESpawnCommand.SAVE_UNPACKED_DIRECTORY)).toHaveLength(0);

    await userEvent.click(getByRole("button", { name: "Export" }));

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_UNPACKED_DIRECTORY)).toEqual([
      { path: EXPORT_PATH, sessionId: service.sessionId },
    ]);
  });

  it("exports nothing when the confirmation is cancelled", async () => {
    mockOpen.mockResolvedValue(EXPORT_PATH);

    const {
      render: { findByRole, getByRole },
    }: IActionsRender = await renderActions();

    await userEvent.click(getByRole("button", { name: "Export spawn file" }));
    await findByRole("dialog");
    await userEvent.click(getByRole("button", { name: "Cancel" }));

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_UNPACKED_DIRECTORY)).toHaveLength(0);
  });

  it("asks nothing further when no export directory was picked", async () => {
    const {
      render: { getByRole, queryByRole },
    }: IActionsRender = await renderActions();

    await userEvent.click(getByRole("button", { name: "Export spawn file" }));

    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("holds both actions while a write runs", async () => {
    const answer = deferred<null>();

    mockSave.mockResolvedValue(SAVE_PATH);

    const {
      render: { getByRole },
    }: IActionsRender = await renderActions();

    mockIpcCommands({ [ESpawnCommand.SAVE_FILE]: () => answer.promise });

    await userEvent.click(getByRole("button", { name: "Save spawn file" }));

    expect(getByRole("button", { name: "Save spawn file" })).toBeDisabled();
    expect(getByRole("button", { name: "Export spawn file" })).toBeDisabled();

    await act(async () => answer.resolve(null));

    expect(getByRole("button", { name: "Save spawn file" })).toBeEnabled();
  });
});
