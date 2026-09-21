import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { LevelLoadService } from "@/core/level/services";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A table holding a scripted wall mark, an ordinary surface, and an entry naming nothing. */
const TABLE: Array<XraySurfaceDescriptor> = [
  mockSurfaceDescriptor({ shader: "default" }),
  mockSurfaceDescriptor({ shader: null, declaration: { kind: "undeclared" } }),
  mockSurfaceDescriptor({
    declaration: {
      function: "normal",
      isAlphaTested: true,
      isBlended: true,
      isDepthWritten: false,
      isWallmark: true,
      kind: "scripted",
      script: "shaders\\r2\\effects_wallmarkmult.s",
    },
    draw: { isDoubled: true, kind: "multiplied" },
    shader: "effects\\wallmarkmult",
  }),
];

async function renderPanel(): Promise<RenderResult> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription({ surfaces: TABLE })),
  });

  const container: Container = mockContainer([LevelLoadService]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.restore();

  return renderWithProviders(<LevelSurfacesPanel />, { container, route: "/level-viewer" });
}

describe("LevelSurfacesPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("says nothing is open before a level is", () => {
    const container: Container = mockContainer([LevelLoadService]);
    const { getByText } = renderWithProviders(<LevelSurfacesPanel />, { container, route: "/level-viewer" });

    expect(getByText("No level open. Open one to see how its surfaces are drawn.")).toBeInTheDocument();
  });

  // The whole reason this panel exists: telling a shader read from a renderer script apart from one read as a
  // blender class, since the two draw differently and nothing else in the viewer says which happened.
  it("names each entry and what it was read from", async () => {
    const { getByText } = await renderPanel();

    expect(getByText("2 · effects\\wallmarkmult")).toBeInTheDocument();
    expect(getByText(/shaders\\r2\\effects_wallmarkmult\.s, function 'normal'/)).toBeInTheDocument();
    expect(getByText("Multiplied 2x")).toBeInTheDocument();
  });

  it("numbers a row by the shader id every surface refers to it by", async () => {
    const { getByText } = await renderPanel();

    expect(getByText("0 · default")).toBeInTheDocument();
  });

  // The table keeps the places of the entries naming nothing, so the count is said once rather than listed.
  it("counts the whole table and lists only what names a shader", async () => {
    const { getByText, queryByText } = await renderPanel();

    expect(getByText("3")).toBeInTheDocument();
    expect(getByText("2")).toBeInTheDocument();
    expect(queryByText("1 · (no shader)")).not.toBeInTheDocument();
  });
});
