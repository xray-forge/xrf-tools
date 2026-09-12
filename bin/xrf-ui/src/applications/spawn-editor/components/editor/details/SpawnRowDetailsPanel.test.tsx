import { describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";

import { SpawnRowDetailsPanel } from "@/applications/spawn-editor/components/editor/details/SpawnRowDetailsPanel";
import { SpawnFileService } from "@/core/spawn/services";
import { mockAlifeObject } from "@/fixtures/mocks/spawn.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AnyObject } from "@/lib/types/general";

interface IPanelRender {
  render: RenderResult;
  service: SpawnFileService;
}

/**
 * Renders the details panel with an optional selected row.
 *
 * Awaited because the panel resolves its own service: providing the container provisions it, and the restore that
 * starts there settles after the first paint.
 *
 * @param selected - Spawn row to select before rendering.
 * @returns Render result and the backing spawn-file service.
 */
async function renderPanel(selected?: AnyObject): Promise<IPanelRender> {
  const { container, service } = mockInjectedService(SpawnFileService);
  const render: RenderResult = await act(async () => renderWithProviders(<SpawnRowDetailsPanel />, { container }));

  // Selected after provisioning, not before: the restore that provisioning starts finds no open file and drops
  // whatever was selected into it, which is the behaviour the last case here asserts.
  if (selected) {
    act(() => service.selectRow("Alife object", 0, selected));
  }

  return { render, service };
}

describe("SpawnRowDetailsPanel", () => {
  it("asks for a selection rather than showing an empty frame", async () => {
    const { render }: IPanelRender = await renderPanel();

    expect(render.getByText("Nothing selected")).toBeInTheDocument();
  });

  it("names what kind of row is showing", async () => {
    const { render }: IPanelRender = await renderPanel(mockAlifeObject());

    expect(render.getByText("Alife object")).toBeInTheDocument();
  });

  it("shows the fields the table columns deliberately leave out", async () => {
    const { render }: IPanelRender = await renderPanel(mockAlifeObject());

    // `inherited` and `updateData` are not columns, which is the point of the panel.
    expect(render.getByText("inherited")).toBeInTheDocument();
    expect(render.getByText("updateData")).toBeInTheDocument();
  });

  it("renders a vector readably rather than as JSON", async () => {
    const { render }: IPanelRender = await renderPanel(mockAlifeObject());

    expect(render.getByText("x: 12.5, y: 1.25, z: -30")).toBeInTheDocument();
  });

  it("says an empty list is empty rather than printing nothing", async () => {
    const { render }: IPanelRender = await renderPanel(mockAlifeObject());

    expect(render.getByText("empty")).toBeInTheDocument();
  });

  it("drops the selection when the file it pointed into closes", async () => {
    const { render, service }: IPanelRender = await renderPanel(mockAlifeObject());

    await act(() => service.closeFile());

    // A selection outliving its data would render a row from a file that is no longer open.
    expect(render.getByText("Nothing selected")).toBeInTheDocument();
  });
});
