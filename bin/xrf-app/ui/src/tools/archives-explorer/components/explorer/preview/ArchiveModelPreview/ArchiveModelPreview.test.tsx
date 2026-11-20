import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { listMockInvokeCalls, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { Nullable } from "@xrf/types";

import { mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSelectedVisual } from "@/core/visuals/fixtures/visual.mocks";
import { VisualLoadService } from "@/core/visuals/services";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { container as ARCHIVES_EXPLORER_CONTAINER } from "@/tools/archives-explorer/runtime";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

import { ArchiveModelPreview } from "./ArchiveModelPreview";

function mockExplorerContainer(): Container {
  const container: Container = mockContainer([...(ARCHIVES_EXPLORER_CONTAINER.bindings ?? [])]);

  mockHeldSessionRoot(container.get(ArchivesService).subject, mockSessionSnapshot(mockArchivesVolumes([])));

  return container;
}

describe("ArchiveModelPreview", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockIpcCommands({
      [ERenderCommand.ATTACH_VIEWPORT]: 1,
      [EVisualsCommand.OPEN_MODEL]: mockSessionResponse(mockSelectedVisual()),
    });
  });

  it("keeps the shown model on screen while the next one opens, and lets it go when unmounted", async () => {
    const container: Container = mockExplorerContainer();
    const { rerender, unmount } = renderWithProviders(<ArchiveModelPreview name={"meshes\\first.ogf"} />, {
      container,
    });

    await act(flush);

    rerender(<ArchiveModelPreview name={"meshes\\second.ogf"} />);

    await act(flush);

    const shown: Array<Nullable<string>> = listMockInvokeCalls(EVisualsCommand.SHOW_MODEL).map(
      ({ sessionId }) => sessionId as Nullable<string>
    );
    const firstShown: number = shown.findIndex(Boolean);

    // An empty scene between the two is the viewport blinking as the tree moves.
    expect(firstShown).toBeGreaterThanOrEqual(0);
    expect(shown.slice(firstShown)).not.toContain(null);
    expect(new Set(shown.slice(firstShown)).size).toBe(2);

    unmount();
    await flush();

    expect(container.get(VisualLoadService).sessionId).toBeNull();
  });
});
