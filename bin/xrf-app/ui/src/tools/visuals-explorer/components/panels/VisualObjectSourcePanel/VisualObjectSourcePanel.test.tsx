import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { AssetSource } from "@xrf/ipc/types/xrf-app-core";

import { mockSelectedObject, mockSelectedVisual } from "@/core/visuals/fixtures/visual.mocks";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualMotionService } from "@/core/visuals/services/visual-motion";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { VisualsService } from "@/tools/visuals-explorer/services/visuals";

import { VisualObjectSourcePanel } from "./VisualObjectSourcePanel";

/** The panel over a fresh explorer that has opened `path`, an object described as the mock object. */
async function renderPanel(path: string): Promise<RenderResult> {
  mockIpcCommands({
    [EVisualsCommand.OPEN_MODEL]: mockSessionResponse((parameters?: Record<string, unknown>) => {
      const source: AssetSource = (parameters as { source: AssetSource }).source;

      return describeVisualSource(source).endsWith(".object")
        ? mockSelectedVisual({ source, object: mockSelectedObject() })
        : mockSelectedVisual({ source });
    }),
  });

  const container: Container = mockContainer([VisualLoadService, VisualMotionService, VisualsService]);

  await container.get(VisualsService).openFile(path);

  return renderWithProviders(<VisualObjectSourcePanel />, { container });
}

describe("VisualObjectSourcePanel", () => {
  it("shows what only the object holds and where it was decompiled from", async () => {
    const render: RenderResult = await renderPanel("C:\\rawdata\\crate.object");

    expect(render.getByText("C:\\gamedata\\meshes\\dynamics\\crate.ogf")).toBeInTheDocument();
    expect(render.getByText(/Compiler shaders: the Actor Editor's `default`/)).toBeInTheDocument();
    expect(render.getByText("Box shape, Rigid joint, objects\\wood, 10 kg")).toBeInTheDocument();
  });

  it("asks for an object where a compiled model is open", async () => {
    const render: RenderResult = await renderPanel("C:\\gamedata\\meshes\\wpn_ak74.ogf");

    expect(render.getByText(/No SDK object open/)).toBeInTheDocument();
  });
});
