import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { AsyncState } from "@xrf/async";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";
import { EPathEntryKind } from "@xrf/path";

import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockSessionRead, mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import {
  IArchivesExplorerMock,
  mockArchivesExplorer,
} from "@/tools/archives-explorer/fixtures/archives-explorer.mocks";

import { ArchivesMenu } from "./ArchivesMenu";

interface IRenderedMenu {
  render: RenderResult;
  explorer: IArchivesExplorerMock;
}

async function renderMenu(files: Array<ArchiveFileDescriptor>): Promise<IRenderedMenu> {
  mockIpcCommands({
    [EArchivesCommand.RESTORE_SUBJECT]: mockSessionResponse(mockArchivesVolumes(files)),
    [EArchivesCommand.LIST_OVERRIDES]: mockSessionRead({ overridden: [], unreachable: [] }),
    [EArchivesCommand.LIST_SHARED_PAYLOADS]: mockSessionRead([]),
    [EArchivesCommand.READ_FILE]: mockSessionRead({ name: files[0]?.name ?? "", content: "[system]", size: 8 }),
  });

  const explorer: IArchivesExplorerMock = mockArchivesExplorer();

  const render = renderWithProviders(<ArchivesMenu />, { container: explorer.container });

  await waitFor(() => expect(explorer.archivesService.subject.isRestored).toBe(true));

  return { explorer, render };
}

function fileRow(render: RenderResult, label: string): HTMLElement {
  return render.getByText(label).closest("[role='treeitem']") as HTMLElement;
}

describe("ArchivesMenu", () => {
  it("selects a file on one click without opening it", async () => {
    const { render, explorer } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.click(await render.findByText("system.ltx"));

    expect(explorer.previewService.selectedEntry).toBeNull();
    expect(fileRow(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("opens a file on a double click", async () => {
    const { render, explorer } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.dblClick(await render.findByText("system.ltx"));

    expect(explorer.previewService.selectedEntry?.name).toBe("configs\\system.ltx");
  });

  it("takes a directory as the extraction target on a double click, and opens it", async () => {
    const { render, explorer } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.click(render.getByText("configs"));

    expect(explorer.previewService.selection).toEqual({ kind: "none" });

    fireEvent.dblClick(render.getByText("configs"));

    expect(explorer.previewService.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });
    expect(await render.findByText("system.ltx")).toBeInTheDocument();
  });

  it("keeps selecting free while a read is in flight", async () => {
    const { render, explorer } = await renderMenu([
      mockArchiveFileDescriptor({ name: "configs\\system.ltx" }),
      mockArchiveFileDescriptor({ name: "configs\\game.ltx" }),
    ]);

    fireEvent.dblClick(render.getByText("configs"));

    await act(() => mockCommandAnswer(explorer.previewService["read"], AsyncState.loading()));

    fireEvent.click(await render.findByText("game.ltx"));

    // Selecting is inert, so it never waits on a read.
    expect(fileRow(render, "game.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("blocks Enter on search results during extraction and opens them after it finishes", async () => {
    const { render, explorer } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);
    const input = render.getByRole("textbox", { name: "Filter archive files" });

    fireEvent.change(input, { target: { value: "system" } });
    await act(() => mockCommandAnswer(explorer.archivesService.fileExtraction, AsyncState.loading()));
    fireEvent.keyDown(input, { key: "Enter" });

    expect(explorer.previewService.selectedEntry).toBeNull();

    fireEvent.change(input, { target: { value: "missing" } });

    expect(render.getByText("No files match missing.")).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "system" } });
    await act(() => mockCommandAnswer(explorer.archivesService.fileExtraction, AsyncState.idle()));
    fireEvent.keyDown(input, { key: "Enter" });

    expect(explorer.previewService.selectedEntry?.name).toBe("configs\\system.ltx");

    fireEvent.change(input, { target: { value: "" } });

    expect(fileRow(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("blocks clicks on search results during extraction and opens them after it finishes", async () => {
    const { render, explorer } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.change(render.getByRole("textbox", { name: "Filter archive files" }), { target: { value: "system" } });
    await act(() => mockCommandAnswer(explorer.archivesService.fileExtraction, AsyncState.loading()));

    const result = render.getByRole("button", { name: "system.ltx configs" });

    expect(result).toHaveAttribute("aria-disabled", "true");

    fireEvent.click(result);

    expect(explorer.previewService.selectedEntry).toBeNull();

    await act(() => mockCommandAnswer(explorer.archivesService.fileExtraction, AsyncState.idle()));
    fireEvent.click(render.getByRole("button", { name: "system.ltx configs" }));

    expect(explorer.previewService.selectedEntry?.name).toBe("configs\\system.ltx");
  });
});
