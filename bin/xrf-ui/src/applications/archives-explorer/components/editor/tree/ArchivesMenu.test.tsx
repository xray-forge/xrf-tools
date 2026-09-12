import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ArchivesMenu } from "@/applications/archives-explorer/components/editor/tree/ArchivesMenu";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

interface IRenderedMenu {
  render: RenderResult;
  container: Container;
  service: ArchivesService;
}

/** Renders the menu after the service restores its open project. */
async function renderMenu(files: Array<ArchiveFileDescriptor>): Promise<IRenderedMenu> {
  setMockInvokeResponses({
    ["plugin:archives|get_project"]: mockSessionResponse(mockArchivesProject(files)),
    ["plugin:archives|list_collisions"]: [],
    ["plugin:archives|list_shared_payloads"]: [],
    ["plugin:archives|read_file"]: { name: files[0]?.name ?? "", content: "[system]", size: 8 },
  });

  const { container, service } = mockInjectedService(ArchivesService);

  const render = renderWithProviders(<ArchivesMenu />, { container });

  await waitFor(() => expect(service.isReady).toBe(true));

  return { container, service, render };
}

function fileRow(render: RenderResult, label: string): HTMLElement {
  return render.getByText(label).closest("[role='treeitem']") as HTMLElement;
}

describe("ArchivesMenu", () => {
  it("selects a file on one click without reading it", async () => {
    const { render } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.click(await render.findByText("system.ltx"));

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(fileRow(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("reads a file on a double click", async () => {
    const { render } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.dblClick(await render.findByText("system.ltx"));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", {
        sessionId: expect.any(String),
        path: "configs\\system.ltx",
      })
    );
  });

  it("takes a directory as the extraction target on a double click, and opens it", async () => {
    const { render, service } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.click(render.getByText("configs"));

    expect(service.selectedDirectory).toBeNull();

    fireEvent.dblClick(render.getByText("configs"));

    expect(service.selectedDirectory).toBe("configs");
    expect(await render.findByText("system.ltx")).toBeInTheDocument();
  });

  it("keeps browsing free while a read is in flight, and supersedes it with the next open", async () => {
    const { render, service } = await renderMenu([
      mockArchiveFileDescriptor({ name: "configs\\system.ltx" }),
      mockArchiveFileDescriptor({ name: "configs\\game.ltx" }),
    ]);

    fireEvent.dblClick(render.getByText("configs"));

    act(() => runInAction(() => (service.content = service.content.asLoading(null))));

    fireEvent.click(await render.findByText("game.ltx"));

    // Selecting is inert, so it never waits on a read.
    expect(fileRow(render, "game.ltx")).toHaveAttribute("aria-selected", "true");

    fireEvent.dblClick(render.getByText("game.ltx"));

    // The read in flight is abandoned for this one rather than swallowing the gesture that replaces it.
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", {
        sessionId: expect.any(String),
        path: "configs\\game.ltx",
      })
    );
  });

  it("refuses to open anything while an extraction is writing to disk", async () => {
    const { render, service } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));

    // A write leaves the archive and cannot be abandoned the way a read can, so it still holds an open back.
    act(() => runInAction(() => (service.operation = service.operation.asLoading(null))));

    fireEvent.dblClick(await render.findByText("system.ltx"));

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
  });
});
