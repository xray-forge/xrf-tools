import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ArchivesMenu } from "@/applications/archives-explorer/components/editor/tree/ArchivesMenu";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { createLoadable } from "@/lib/loadable";

interface IRenderedMenu {
  render: RenderResult;
  container: Container;
  service: ArchivesService;
}

/** Renders the menu over a service that has already opened a project. */
function renderMenu(files: Array<ArchiveFileDescriptor>): IRenderedMenu {
  setMockInvokeResponses({
    ["plugin:archives|read_file"]: { name: files[0]?.name ?? "", content: "[system]", size: 8 },
  });

  const { container, service } = mockInjectedService(ArchivesService);

  service.project = createLoadable(mockArchivesProject(files));

  return { container, service, render: renderWithProviders(<ArchivesMenu />, { container }) };
}

function fileRow(render: RenderResult, label: string): HTMLElement {
  return render.getByText(label).closest("[role='treeitem']") as HTMLElement;
}

describe("ArchivesMenu", () => {
  it("selects a file on one click without reading it", async () => {
    const { render } = renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.click(await render.findByText("system.ltx"));

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(fileRow(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("reads a file on a double click", async () => {
    const { render } = renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.dblClick(await render.findByText("system.ltx"));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", { path: "configs\\system.ltx" })
    );
  });

  it("takes a directory as the extraction target on a double click, and opens it", async () => {
    const { render, service } = renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.click(render.getByText("configs"));

    expect(service.selectedDirectory).toBeNull();

    fireEvent.dblClick(render.getByText("configs"));

    expect(service.selectedDirectory).toBe("configs");
    expect(await render.findByText("system.ltx")).toBeInTheDocument();
  });

  it("keeps browsing free while a read is in flight, and refuses to start another", async () => {
    const { render, service } = renderMenu([
      mockArchiveFileDescriptor({ name: "configs\\system.ltx" }),
      mockArchiveFileDescriptor({ name: "configs\\game.ltx" }),
    ]);

    fireEvent.dblClick(render.getByText("configs"));

    act(() => runInAction(() => (service.content = createLoadable(null, true))));

    fireEvent.click(await render.findByText("game.ltx"));

    // Selecting is inert, so it never waits on a read; opening is what the busy state holds back.
    expect(fileRow(render, "game.ltx")).toHaveAttribute("aria-selected", "true");

    fireEvent.dblClick(render.getByText("game.ltx"));

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
  });
});
