import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { LevelLoadService } from "@/core/level/services";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

/** A table holding a scripted wall mark, an ordinary surface, and an entry naming nothing. */
const TABLE: Array<XraySurfaceDescriptor> = [
  mockSurfaceDescriptor({ shader: "default", textures: ["wall\\wall_panel"] }),
  mockSurfaceDescriptor({ shader: null, declaration: { kind: "undeclared" }, textures: [] }),
  mockSurfaceDescriptor({
    declaration: {
      function: "normal",
      alphaReference: 0,
      isAlphaTested: true,
      isBlended: true,
      isDepthWritten: false,
      isWallmark: true,
      kind: "scripted",
      script: "shaders\\r2\\effects_wallmarkmult.s",
    },
    draw: { isDoubled: true, kind: "multiplied" },
    shader: "effects\\wallmarkmult",
    textures: ["decal\\decal_poteki"],
  }),
  // The same shader again, which a level names once per decal: five of these is what Pripyat holds.
  mockSurfaceDescriptor({
    declaration: {
      function: "normal",
      alphaReference: 0,
      isAlphaTested: true,
      isBlended: true,
      isDepthWritten: false,
      isWallmark: true,
      kind: "scripted",
      script: "shaders\\r2\\effects_wallmarkmult.s",
    },
    draw: { isDoubled: true, kind: "multiplied" },
    shader: "effects\\wallmarkmult",
    textures: ["decal\\decal_rza_a"],
  }),
];

async function renderPanel(textures?: ILevelTextureLookup): Promise<RenderResult> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription({ surfaces: TABLE })),
  });

  const container: Container = mockContainer([LevelLoadService]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.restore();

  // The set a level fills by streaming, which no test here streams: given one, the panel reads it instead.
  if (textures) {
    jest.spyOn(service, "textures", "get").mockReturnValue(textures);
  }

  return renderWithProviders(<LevelSurfacesPanel />, { container, route: "/level-viewer" });
}

function mockTextures(entries: Record<string, ILevelTexture>): ILevelTextureLookup {
  return {
    get: (reference: string): Nullable<ILevelTexture> => entries[reference] ?? null,
    listProblems: () => [],
    size: Object.keys(entries).length,
  };
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
    const { getByText, getAllByText } = await renderPanel();

    expect(getByText("2 · effects\\wallmarkmult · decal\\decal_poteki")).toBeInTheDocument();
    expect(getAllByText(/shaders\\r2\\effects_wallmarkmult\.s, function 'normal'/)).toHaveLength(2);
    expect(getAllByText("Multiplied 2x")).toHaveLength(2);
  });

  it("numbers a row by the shader id every surface refers to it by", async () => {
    const { getByText } = await renderPanel();

    expect(getByText("0 · default · wall\\wall_panel")).toBeInTheDocument();
  });

  // A level's table holds one entry per shader **and** texture set, so the shader alone leaves five wall marks as
  // five rows nobody can tell apart. The texture is what distinguishes them.
  it("tells two entries of one shader apart by what they dress with", async () => {
    const { getByText } = await renderPanel();

    expect(getByText("2 · effects\\wallmarkmult · decal\\decal_poteki")).toBeInTheDocument();
    expect(getByText("3 · effects\\wallmarkmult · decal\\decal_rza_a")).toBeInTheDocument();
  });

  // A surface drawn from a checker looks exactly like a blending fault and is not one. The entry says what the level
  // asked for; without this the panel never says what the renderer actually got.
  it("says when a checker stands in for what a row dresses with", async () => {
    const { getByText } = await renderPanel(
      mockTextures({
        ["decal\\decal_poteki"]: {
          isAlphaRead: true,
          reason: "Nothing in the mounted roots answers to it",
          texture: null,
        },
      })
    );

    expect(
      getByText("decal\\decal_poteki · a checker stands in: Nothing in the mounted roots answers to it")
    ).toBeInTheDocument();
  });

  it("says a texture no resident sector has asked for has not been read", async () => {
    const { getByText } = await renderPanel();

    expect(getByText("wall\\wall_panel · not read yet")).toBeInTheDocument();
  });

  // The shape of the geometry is what a shader table entry cannot say, and it is what tells a mark that follows a
  // surface from a quad laid over it. Nothing is resident in this test, so every row says so rather than nothing.
  it("says how much geometry each entry draws", async () => {
    const { getAllByText } = await renderPanel();

    expect(getAllByText("nothing resident draws it")).toHaveLength(3);
  });

  // The table keeps the places of the entries naming nothing, so the count is said once rather than listed.
  it("counts the whole table and lists only what names a shader", async () => {
    const { getByText, queryByText } = await renderPanel();

    expect(getByText("4")).toBeInTheDocument();
    expect(getByText("3")).toBeInTheDocument();
    expect(queryByText(/\(no shader\)/)).not.toBeInTheDocument();
  });
});
