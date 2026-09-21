import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/ipc/types/xrf-archive";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchivesMenu } from "./ArchivesMenu";

interface IRenderedMenu {
  render: RenderResult;
  container: Container;
  service: ArchivesService;
}

async function renderMenu(files: Array<ArchiveFileDescriptor>): Promise<IRenderedMenu> {
  setMockInvokeResponses({
    ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesVolumes(files)),
    ["plugin:archives|list_overrides"]: { overridden: [], unreachable: [] },
    ["plugin:archives|list_shared_payloads"]: [],
    ["plugin:archives|read_file"]: { name: files[0]?.name ?? "", content: "[system]", size: 8 },
  });

  const { container, service } = mockInjectedService(ArchivesService);

  const render = renderWithProviders(<ArchivesMenu />, { container });

  await waitFor(() => expect(service.subject.isReady).toBe(true));

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

    expect(service.selection).toEqual({ kind: "none" });

    fireEvent.dblClick(render.getByText("configs"));

    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });
    expect(await render.findByText("system.ltx")).toBeInTheDocument();
  });

  it("keeps browsing free while a read is in flight, and supersedes it with the next open", async () => {
    const { render, service } = await renderMenu([
      mockArchiveFileDescriptor({ name: "configs\\system.ltx" }),
      mockArchiveFileDescriptor({ name: "configs\\game.ltx" }),
    ]);

    fireEvent.dblClick(render.getByText("configs"));

    await act(() => runInAction(() => (service.content = service.content.asLoading(null))));

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
    await act(() => runInAction(() => (service.operation = service.operation.asLoading(null))));

    fireEvent.dblClick(await render.findByText("system.ltx"));

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
  });

  it("blocks Enter on search results during extraction and opens them after it finishes", async () => {
    const { render, service } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);
    const input = render.getByRole("textbox", { name: "Filter archive files" });

    fireEvent.change(input, { target: { value: "system" } });
    await act(() => runInAction(() => (service.operation = service.operation.asLoading(null))));
    fireEvent.keyDown(input, { key: "Enter" });

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());

    fireEvent.change(input, { target: { value: "missing" } });

    expect(render.getByText("No files match missing.")).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "system" } });
    await act(() => runInAction(() => (service.operation = service.operation.asIdle())));
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", {
        sessionId: expect.any(String),
        path: "configs\\system.ltx",
      })
    );

    fireEvent.change(input, { target: { value: "" } });

    expect(fileRow(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("blocks clicks on search results during extraction and opens them after it finishes", async () => {
    const { render, service } = await renderMenu([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })]);

    fireEvent.change(render.getByRole("textbox", { name: "Filter archive files" }), { target: { value: "system" } });
    await act(() => runInAction(() => (service.operation = service.operation.asLoading(null))));

    const result = render.getByRole("button", { name: "system.ltx configs" });

    expect(result).toHaveAttribute("aria-disabled", "true");

    fireEvent.click(result);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());

    await act(() => runInAction(() => (service.operation = service.operation.asIdle())));
    fireEvent.click(render.getByRole("button", { name: "system.ltx configs" }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", {
        sessionId: expect.any(String),
        path: "configs\\system.ltx",
      })
    );
  });
});
