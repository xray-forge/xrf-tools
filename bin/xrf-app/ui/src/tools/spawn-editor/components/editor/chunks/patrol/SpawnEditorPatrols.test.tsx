import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { ESpawnChunkKind, SpawnChunk } from "@xrf/ipc/types/xrf-app-spawn";

import { mockSessionRead, mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockPatrol, mockSpawnSession, mockVector3d } from "@/tools/spawn-editor/fixtures/spawn.mocks";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

import { SpawnEditorPatrols } from "./SpawnEditorPatrols";

/** Two patrols: the fixture's walk of two linked points, and a single-point guard post with no links. */
const PATROLS: SpawnChunk = {
  kind: ESpawnChunkKind.PATROLS,
  patrols: [
    mockPatrol(),
    mockPatrol({
      name: "esc_guard_post",
      points: [{ name: "wp00", position: mockVector3d(), flags: 0, levelVertexId: 300, gameVertexId: 30 }],
      links: [],
    }),
  ],
};

async function renderPatrols(route: string): Promise<RenderResult> {
  mockIpcCommands({
    [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()),
    [ESpawnCommand.READ_CHUNK]: mockSessionRead(PATROLS),
  });

  const { container, service } = mockInjectedService(SpawnEditorService);

  await service.open("C:\\game\\all.spawn");

  return act(async () => renderWithProviders(<SpawnEditorPatrols />, { container, route }));
}

describe("SpawnEditorPatrols", () => {
  beforeEach(() => mockIpcCommands({}));

  it("lists one row per patrol by default", async () => {
    const { findByTestId } = await renderPatrols("/spawn-editor/patrols");

    expect(await findByTestId("spawn-editor-patrols-table")).toHaveTextContent("2 patrol(s)");
  });

  it("lists the points of every patrol together", async () => {
    const { findByTestId } = await renderPatrols("/spawn-editor/patrols/points");

    expect(await findByTestId("spawn-editor-patrol-points-table")).toHaveTextContent("3 point(s)");
  });

  it("lists the links of every patrol together, leaving out patrols without any", async () => {
    const { findByTestId, queryByTestId } = await renderPatrols("/spawn-editor/patrols/links");

    expect(await findByTestId("spawn-editor-patrol-links-table")).toHaveTextContent("1 link(s)");
    expect(queryByTestId("spawn-editor-patrols-table")).not.toBeInTheDocument();
  });
});
