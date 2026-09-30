import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import {
  LevelLoadService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
  LevelWeatherService,
} from "@/core/level/services";
import { mockLevelTextureReport, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A table holding a scripted wall mark, an ordinary surface, and an entry naming nothing. */
const TABLE: Array<XraySurfaceDescriptor> = [
  mockSurfaceDescriptor({ shader: "default", textures: ["wall\\wall_panel"] }),
  mockSurfaceDescriptor({ shader: null, declaration: { kind: "undeclared" }, textures: [] }),
  mockSurfaceDescriptor({
    declaration: {
      function: "normal",
      program: "wmark",
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
      program: "wmark",
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

async function renderPanel(
  textures?: ILevelTextureReport,
  arrange?: (render: LevelRenderService) => void
): Promise<RenderResult> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription({ surfaces: TABLE })),
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
  ]);
  const service: LevelLoadService = container.get(LevelLoadService);

  // What a level's textures came to, which no test here streams for itself: given one, the panel reads it.
  if (textures) {
    container.get(LevelViewportService).noteTextures(textures);
  }

  arrange?.(container.get(LevelRenderService));

  const result: RenderResult = renderWithProviders(<LevelSurfacesPanel />, { container, route: "/level-viewer" });

  // Mounting provisions the container, and that restores the level the backend holds.
  await waitFor(() => expect(service.isReady).toBe(true));

  return result;
}

/**
 * Chooses the entry a filter finds, which opens its shader.
 *
 * @param view - The panel.
 * @param filter - What finds the entry.
 * @param label - The entry's row, as the tree names it.
 * @returns Its details, under the tree.
 */
async function choose(view: RenderResult, filter: string, label: string): Promise<HTMLElement> {
  await userEvent.type(view.getByRole("textbox", { name: "Filter surfaces" }), filter);
  await userEvent.click(await view.findByText(label));

  return view.findByTestId("level-surface-row");
}

function listRows(view: RenderResult): Array<string> {
  return within(view.getByRole("tree", { name: "Shader table" }))
    .getAllByRole("treeitem")
    .map((it: HTMLElement) => it.textContent ?? "");
}

describe("LevelSurfacesPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("says nothing is open before a level is", () => {
    const container: Container = mockContainer([
      LevelLoadService,
      LevelRenderService,
      LevelViewService,
      LevelViewportService,
      LevelWeatherService,
    ]);
    const { getByText } = renderWithProviders(<LevelSurfacesPanel />, { container, route: "/level-viewer" });

    expect(getByText("No level open. Open one to see how its surfaces are drawn.")).toBeInTheDocument();
  });

  // A level's table holds one entry per shader and texture set, so the shader groups what the texture tells apart; the
  // table keeps the places of the entries naming nothing, which are not listed.
  it("groups the entries naming a shader by it, counting each shader's", async () => {
    const view: RenderResult = await renderPanel();

    expect(listRows(view)).toEqual(["default1", "effects\\wallmarkmult2"]);
    expect(view.getByText("3")).toBeInTheDocument();
    expect(view.queryByText(/\(no shader\)/)).not.toBeInTheDocument();
  });

  // Numbered by the shader id every surface refers to it by, named by what it dresses with, captioned by its draw.
  it("opens what a filter matches, each entry by its id and texture with what it is drawn as", async () => {
    const view: RenderResult = await renderPanel();

    await userEvent.type(view.getByRole("textbox", { name: "Filter surfaces" }), "decal");

    await waitFor(() =>
      expect(listRows(view)).toEqual([
        "effects\\wallmarkmult2",
        "2 · decal\\decal_potekiMultiplied 2x",
        "3 · decal\\decal_rza_aMultiplied 2x",
      ])
    );
  });

  // The whole reason this panel exists: telling a shader read from a renderer script apart from one read as a
  // blender class, since the two draw differently and nothing else in the viewer says which happened.
  it("says what a chosen entry was read from and what it is drawn as", async () => {
    const view: RenderResult = await renderPanel();
    const details: HTMLElement = await choose(view, "poteki", "2 · decal\\decal_poteki");

    expect(details).toHaveTextContent("2 · effects\\wallmarkmult · decal\\decal_poteki");
    expect(within(details).getByText(/shaders\\r2\\effects_wallmarkmult\.s, function 'normal'/)).toBeInTheDocument();
    expect(within(details).getByText("Multiplied 2x")).toBeInTheDocument();
  });

  // Where an entry lands in the frame decides what light it sees, which the draw alone does not say: a wall mark
  // multiplies the albedo before the light arrives, and that is what made its marks read right.
  it("says which pass of the frame draws the entry", async () => {
    const view: RenderResult = await renderPanel();
    const details: HTMLElement = await choose(view, "poteki", "2 · decal\\decal_poteki");

    expect(within(details).getByText("the albedo, before any light reaches it")).toBeInTheDocument();
  });

  // A surface drawn from a checker looks exactly like a blending fault and is not one. The entry says what the level
  // asked for; without this the panel never says what the renderer actually got.
  it("says when a checker stands in for what the entry dresses with", async () => {
    const view: RenderResult = await renderPanel(
      mockLevelTextureReport({
        ["decal\\decal_poteki"]: { reason: "Nothing in the mounted roots answers to it", upload: null },
      })
    );
    const details: HTMLElement = await choose(view, "poteki", "2 · decal\\decal_poteki");

    expect(
      within(details).getByText("decal\\decal_poteki · a checker stands in: Nothing in the mounted roots answers to it")
    ).toBeInTheDocument();
  });

  it("says a texture no resident sector has asked for has not been read", async () => {
    const view: RenderResult = await renderPanel();
    const details: HTMLElement = await choose(view, "wall_panel", "0 · wall\\wall_panel");

    expect(within(details).getByText("wall\\wall_panel · not read yet")).toBeInTheDocument();
    // Nothing is resident in this test, so the entry says so rather than nothing.
    expect(within(details).getByText("nothing resident draws it")).toBeInTheDocument();
  });

  // Measuring what a surface draws samples the coordinates of every draw of every sector held, so it is asked of
  // whoever holds them rather than published, and only once an entry is chosen.
  it("shows what the renderer's content answers when asked what the chosen entry draws", async () => {
    const measure = jest.fn(
      () =>
        new Map(
          [0, 1, 2, 3].map((shaderId) => [shaderId, { drawables: 2, narrowest: null, span: null, triangles: 70 }])
        )
    );
    const view: RenderResult = await renderPanel(undefined, (render: LevelRenderService) => {
      jest.spyOn(render, "measureSurfaceGeometry").mockImplementation(measure);
    });

    expect(measure).not.toHaveBeenCalled();

    const details: HTMLElement = await choose(view, "poteki", "2 · decal\\decal_poteki");

    expect(within(details).getByText("2 drawables · 70 triangles · 35.0 each")).toBeInTheDocument();
  });
});
