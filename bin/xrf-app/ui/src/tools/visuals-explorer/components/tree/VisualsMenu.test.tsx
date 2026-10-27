import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { EAssetsCommand } from "@xrf/ipc/commands/assets";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { XrayAsset, XrayRoots } from "@xrf/ipc/types/xrf-vfs";

import { createRoots } from "@/core/assets/lib";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualMotionService } from "@/core/visuals/services/visual-motion";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { VisualsService } from "@/tools/visuals-explorer/services/visuals";
import { VisualsBrowseService } from "@/tools/visuals-explorer/services/visuals-browse";

import { VisualsMenu } from "./VisualsMenu";

function mockLooseVisual(logicalPath: string): XrayAsset {
  return { container: { kind: "directory", relativePath: logicalPath, root: "C:\\gamedata" }, logicalPath };
}

function mockArchivedVisual(logicalPath: string): XrayAsset {
  return { container: { kind: "archive", path: "C:\\game\\db\\meshes.db0" }, logicalPath };
}

/** Renders the menu over a browse service that has already listed roots. */
async function renderMenu(visuals: Array<XrayAsset>): Promise<{ render: RenderResult; container: Container }> {
  mockIpcCommands({
    [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
    [EAssetsCommand.LIST_ASSETS]: visuals,
    [EVisualsCommand.RESTORE_MODEL]: mockSessionResponse(null),
  });

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

    mockIpcCommands({
      [EVisualsCommand.OPEN_MODEL]: () => {
        openCalls += 1;

        throw new Error("Nothing was meant to open");
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

    mockIpcCommands({
      [EVisualsCommand.OPEN_MODEL]: mockSessionResponse((parameters?: Record<string, unknown>) => {
        openParameters = parameters ?? {};

        throw new Error("stop before geometry");
      }),
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
