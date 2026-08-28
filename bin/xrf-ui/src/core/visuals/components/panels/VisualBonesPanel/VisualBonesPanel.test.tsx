import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, within } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { ProjectService } from "@/core/settings/services/project";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualBonesPanel } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBonesPanel";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockVisualBone,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

async function renderPanel(): Promise<{ render: RenderResult; service: VisualsService }> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const selected: SelectedVisualDescription = mockSelectedVisual({
    description: mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer)],
      bufferLength: buffer.byteLength,
      bones: [
        mockVisualBone({ name: "wpn_body", parent: "" }),
        mockVisualBone({ name: "wpn_scope", parent: "wpn_body" }),
      ],
    }),
  });

  setMockInvokeResponses({
    ["plugin:visuals|open_model"]: selected,
    ["plugin:visuals|read_geometry"]: buffer.toArrayBuffer(),
  });

  const container: Container = mockContainer([
    ProjectService,
    VisualLoadService,
    VisualMotionService,
    VisualsService,
    // The binding an application makes at its composition root, which is what the panel resolves through.
    { token: VISUAL_INSPECTION, factory: (it: Container) => it.get(VisualsService) },
  ]);
  const service: VisualsService = container.get(VisualsService);

  await service.openFile("C:\\gamedata\\wpn_ak74.ogf");

  return { render: renderWithProviders(<VisualBonesPanel />, { container }), service };
}

describe("VisualBonesPanel", () => {
  it("highlights a bone on one click, since drawing the selection is not opening anything", async () => {
    const { render, service } = await renderPanel();

    // Scoped to the tree: an addon bone is also named by its own switch in the visibility section below.
    const label: HTMLElement = within(render.getByRole("tree")).getByText("wpn_scope");

    // Root bones open with the model, so a child is on screen without touching the tree first.
    fireEvent.click(label);

    expect(service.highlightedBone).toBe("wpn_scope");
    expect(label.closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");
  });
});
