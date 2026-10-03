import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MockVisualInspectionService } from "@/fixtures/mocks/visual-inspection.mocks";
import {
  mockMaterialDescriptor,
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { VisualMaterialsPanel } from "./VisualMaterialsPanel";

async function renderPanel(overrides: Partial<SelectedVisualDescription>): Promise<RenderResult> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const selected: SelectedVisualDescription = mockSelectedVisual({
    description: mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer, { textureName: "wpn\\wpn_ak74" })],
      bufferLength: buffer.byteLength,
    }),
    dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
    ...overrides,
  });

  setMockInvokeResponses({
    ["plugin:visuals|open_model"]: mockSessionResponse(selected),
  });

  const container: Container = mockContainer([
    VisualLoadService,
    MockVisualInspectionService,
    { token: VISUAL_INSPECTION, factory: (it: Container) => it.get(MockVisualInspectionService) },
  ]);

  await container.get(MockVisualInspectionService).openFile("C:\\gamedata\\wpn_ak74.ogf");

  return renderWithProviders(<VisualMaterialsPanel />, { container });
}

describe("VisualMaterialsPanel", () => {
  it("joins a submesh to its material by the reference the mesh declares", async () => {
    const { getByText } = await renderPanel({
      materials: { ["wpn\\wpn_ak74"]: mockMaterialDescriptor({ outcome: "dummy" }) },
    });

    expect(getByText("Dummy bump")).toBeInTheDocument();
    expect(getByText(/1 bumped, 1 degraded/)).toBeInTheDocument();
  });

  it("says a textures.ltx is not read only when the roots hold one", async () => {
    const silent = await renderPanel({});

    expect(silent.queryByText(/is not read/)).not.toBeInTheDocument();
    silent.unmount();

    const { getByText } = await renderPanel({
      texturesLtx: {
        container: { kind: "directory", relativePath: "textures\\textures.ltx", root: "C:\\gamedata" },
        logicalPath: "textures\\textures.ltx",
      },
    });

    expect(getByText(/textures\\textures\.ltx may declare more, and is not read/)).toBeInTheDocument();
  });
});
