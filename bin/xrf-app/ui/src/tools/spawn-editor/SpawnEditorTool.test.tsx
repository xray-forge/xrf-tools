import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { IpcCommandMocks, listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";

import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService, mockToolScope } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockSpawnSession } from "@/tools/spawn-editor/fixtures/spawn.mocks";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorTool } from "./SpawnEditorTool";

const SPAWN_FILE: string = "C:\\game\\gamedata\\spawns\\all.spawn";

interface IToolRender {
  render: RenderResult;
  service: SpawnEditorService;
}

/**
 * Renders the tool over the backend answers given, nothing restored unless they say otherwise.
 *
 * Awaited because providing the container provisions the service, whose restore settles after the first paint.
 */
async function renderTool(responses: IpcCommandMocks = {}): Promise<IToolRender> {
  mockIpcCommands({ [ESpawnCommand.RESTORE_FILE]: mockSessionResponse(null), ...responses });

  const { container, service } = mockInjectedService(SpawnEditorService, mockToolScope(EToolId.SPAWN_EDITOR));
  const render: RenderResult = await act(async () =>
    renderWithProviders(<SpawnEditorTool />, { container, route: "/spawn-editor" })
  );

  return { render, service };
}

describe("SpawnEditorTool", () => {
  beforeEach(() => clearLocalStorage());

  it("offers the open form when the backend holds no spawn for the tool", async () => {
    const {
      render: { findByRole, queryByTestId },
    }: IToolRender = await renderTool();

    expect(await findByRole("button", { name: "Open" })).toBeInTheDocument();
    expect(queryByTestId("spawn-editor-workspace")).not.toBeInTheDocument();
  });

  it("comes back to the spawn the backend still holds, on its header", async () => {
    const {
      render: { findByTestId, queryByRole },
    }: IToolRender = await renderTool({
      [ESpawnCommand.RESTORE_FILE]: mockSessionSnapshot(mockSpawnSession({ path: SPAWN_FILE }), "restored"),
    });

    expect(await findByTestId("spawn-editor-header-table")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Open" })).not.toBeInTheDocument();
  });

  it("opens the spawn file the form names", async () => {
    setLocalStorageValue("xrf.form.spawn-editor.file", SPAWN_FILE);

    const {
      render: { findByRole, findByTestId },
    }: IToolRender = await renderTool({
      [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession({ path: SPAWN_FILE })),
    });
    const submit = await findByRole("button", { name: "Open" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByTestId("spawn-editor-workspace")).toBeInTheDocument();
    expect(listMockInvokeCalls(ESpawnCommand.OPEN_FILE)).toEqual([{ sessionId: expect.any(String), path: SPAWN_FILE }]);
  });

  it("stays on the form and says why a spawn file could not be opened", async () => {
    setLocalStorageValue("xrf.form.spawn-editor.file", SPAWN_FILE);

    const {
      render: { findByRole, findByText, queryByTestId },
    }: IToolRender = await renderTool({
      [ESpawnCommand.OPEN_FILE]: () => {
        throw new Error("not a spawn file");
      },
    });
    const submit = await findByRole("button", { name: "Open" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText(/not a spawn file/)).toBeInTheDocument();
    expect(queryByTestId("spawn-editor-workspace")).not.toBeInTheDocument();
  });

  it("returns to the open form once the open spawn is closed", async () => {
    const {
      render: { findByRole, getByRole },
      service,
    }: IToolRender = await renderTool({
      [ESpawnCommand.RESTORE_FILE]: mockSessionSnapshot(mockSpawnSession({ path: SPAWN_FILE }), "restored"),
    });

    await findByRole("button", { name: "Back to Spawn editor" });
    fireEvent.click(getByRole("button", { name: "Back to Spawn editor" }));

    expect(await findByRole("button", { name: "Open" })).toBeInTheDocument();
    expect(service.isOpen).toBe(false);
    expect(listMockInvokeCalls(ESpawnCommand.CLOSE_FILE)).toEqual([{ sessionIds: ["restored"] }]);
  });
});
