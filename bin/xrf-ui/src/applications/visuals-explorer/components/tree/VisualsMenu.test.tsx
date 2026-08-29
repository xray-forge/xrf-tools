import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { VisualsMenu } from "@/applications/visuals-explorer/components/tree/VisualsMenu";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { createRoots } from "@/core/assets/lib";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

function mockLooseVisual(logicalPath: string): XrayAsset {
  return { container: { kind: "directory", relativePath: logicalPath, root: "C:\\gamedata" }, logicalPath };
}

function mockArchivedVisual(logicalPath: string): XrayAsset {
  return { container: { kind: "archive", path: "C:\\game\\db\\meshes.db0" }, logicalPath };
}

/** Renders the menu over a browse service that has already listed roots. */
async function renderMenu(visuals: Array<XrayAsset>): Promise<{ render: RenderResult; container: Container }> {
  setMockInvokeResponses({ ["plugin:assets|list_assets"]: visuals, ["plugin:visuals|get_model"]: null });

  const container: Container = mockContainer([
    VisualLoadService,
    VisualMotionService,
    VisualsBrowseService,
    VisualsService,
  ]);

  await container.get(VisualsBrowseService).openRoot("C:\\gamedata");

  return { container, render: renderWithProviders(<VisualsMenu />, { container }) };
}

describe("VisualsMenu", () => {
  it("renders the listing as a tree", async () => {
    const { render } = await renderMenu([
      mockLooseVisual("meshes\\actors\\stalker.ogf"),
      mockLooseVisual("meshes\\wpn\\wpn_ak74.ogf"),
    ]);

    expect(render.getByText("meshes")).toBeInTheDocument();
    expect(render.getByRole("heading", { name: "Visuals" })).toBeInTheDocument();
  });

  it("marks a visual that came out of an archive", async () => {
    const { render } = await renderMenu([
      mockLooseVisual("meshes\\wpn\\wpn_ak74.ogf"),
      mockArchivedVisual("meshes\\wpn\\wpn_abakan.ogf"),
    ]);

    fireEvent.dblClick(render.getByText("meshes"));
    fireEvent.dblClick(await render.findByText("wpn"));

    expect(await render.findByText("wpn_abakan.ogf")).toBeInTheDocument();
    expect(render.getAllByText("db")).toHaveLength(1);
  });

  it("selects a visual on one click without opening it", async () => {
    const { render } = await renderMenu([mockLooseVisual("meshes\\actors\\stalker.ogf")]);

    let openCalls: number = 0;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: () => {
        openCalls += 1;

        return null;
      },
    });

    fireEvent.dblClick(render.getByText("meshes"));
    fireEvent.dblClick(await render.findByText("actors"));
    fireEvent.click(await render.findByText("stalker.ogf"));

    expect(openCalls).toBe(0);
    expect(render.getByText("stalker.ogf").closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");
  });

  it("opens the visual a leaf names, in the browsed roots, and keeps it selected when the open fails", async () => {
    const { container, render } = await renderMenu([mockLooseVisual("meshes\\actors\\stalker.ogf")]);

    let openParameters: Record<string, unknown> = {};

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: (parameters?: Record<string, unknown>) => {
        openParameters = parameters ?? {};

        throw new Error("stop before geometry");
      },
    });

    fireEvent.dblClick(render.getByText("meshes"));
    fireEvent.dblClick(await render.findByText("actors"));
    fireEvent.dblClick(await render.findByText("stalker.ogf"));

    await waitFor(() => expect(openParameters.source).toBeDefined());

    expect(openParameters).toMatchObject({
      source: { kind: "asset", logicalPath: "meshes\\actors\\stalker.ogf" },
      roots: createRoots(["C:\\gamedata"]),
    });

    await waitFor(() => expect(container.get(VisualsService).visual.error?.message).toBe("stop before geometry"));

    // Selection is the user's, not the viewport's: a model that failed to load leaves its row selected, beside
    // the retry the failure itself offers.
    expect(render.getByText("stalker.ogf").closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");
  });
});
