import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { IpcCommandAnswer, listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";
import { ESpawnChunkKind, SpawnChunk } from "@xrf/ipc/types/xrf-app-spawn";
import { SpawnGraphsChunk } from "@xrf/ipc/types/xrf-spawn";

import { mockSessionRead, mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockSpawnFile, mockSpawnSession } from "@/tools/spawn-editor/fixtures/spawn.mocks";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorGraphs } from "./SpawnEditorGraphs";

const GRAPHS: SpawnChunk = { kind: ESpawnChunkKind.GRAPHS, ...mockSpawnFile().graphs };

/**
 * Renders the graphs chunk of an open spawn at a route, its reads answered as given.
 *
 * @param route - Where the chunk is shown.
 * @param read - How `read_chunk` answers.
 * @returns The render.
 */
async function renderGraphs(
  route: string,
  read: IpcCommandAnswer<ESpawnCommand.READ_CHUNK> = mockSessionRead(GRAPHS)
): Promise<RenderResult> {
  mockIpcCommands({
    [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()),
    [ESpawnCommand.READ_CHUNK]: read,
  });

  const { container, service } = mockInjectedService(SpawnEditorService);

  await service.open("C:\\game\\all.spawn");

  return act(async () => renderWithProviders(<SpawnEditorGraphs />, { container, route }));
}

describe("SpawnEditorGraphs", () => {
  beforeEach(() => mockIpcCommands({}));

  it("reads the graphs once shown and opens on the table the route names", async () => {
    const { findByTestId } = await renderGraphs("/spawn-editor/graph/levels");

    expect(await findByTestId("spawn-editor-graph-levels-table")).toBeInTheDocument();
    expect(listMockInvokeCalls(ESpawnCommand.READ_CHUNK)).toEqual([
      { sessionId: expect.any(String), chunk: ESpawnChunkKind.GRAPHS },
    ]);
  });

  it("opens on the graph header when the route names no table", async () => {
    const { findByTestId } = await renderGraphs("/spawn-editor/graph");

    expect(await findByTestId("spawn-editor-graph-header-table")).toBeInTheDocument();
  });

  it("shows the table of the tab picked, and that one alone", async () => {
    const { findByTestId, getByRole, queryByTestId } = await renderGraphs("/spawn-editor/graph/levels");

    await findByTestId("spawn-editor-graph-levels-table");
    await userEvent.click(getByRole("tab", { name: "Cross tables" }));

    expect(await findByTestId("spawn-editor-graph-cross-table")).toBeInTheDocument();
    expect(queryByTestId("spawn-editor-graph-levels-table")).not.toBeInTheDocument();
  });

  it("names the pending read while the graphs are on their way", async () => {
    const answer = deferred<SpawnGraphsChunk>();
    const { getByRole } = await renderGraphs(
      "/spawn-editor/graph",
      mockSessionRead(async () => ({ kind: ESpawnChunkKind.GRAPHS, ...(await answer.promise) }) as SpawnChunk)
    );

    expect(getByRole("status")).toHaveTextContent("Reading spawn chunk…");

    await act(async () => answer.resolve(mockSpawnFile().graphs));
  });

  it("offers to read a chunk again after a failed read, and shows it once the retry succeeds", async () => {
    const { findByRole, findByTestId, getByRole } = await renderGraphs("/spawn-editor/graph", () => {
      throw new Error("graph chunk is truncated");
    });

    expect(await findByRole("alert")).toHaveTextContent("Could not read this chunk");
    expect(getByRole("alert")).toHaveTextContent("graph chunk is truncated");

    mockIpcCommands({ [ESpawnCommand.READ_CHUNK]: mockSessionRead(GRAPHS) });
    await userEvent.click(getByRole("button", { name: "Retry" }));

    expect(await findByTestId("spawn-editor-graph-header-table")).toBeInTheDocument();
    expect(listMockInvokeCalls(ESpawnCommand.READ_CHUNK)).toHaveLength(2);
  });
});
