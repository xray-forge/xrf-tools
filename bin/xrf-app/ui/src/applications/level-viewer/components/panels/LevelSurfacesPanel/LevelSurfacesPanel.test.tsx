import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container, EventBus } from "@wirestate/core";
import { AsyncState } from "@xrf/async";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { LEVEL_SURFACES_PART } from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { Nullable } from "@xrf/types";

import { LevelSurfacesPanel } from "@/applications/level-viewer/components/panels/LevelSurfacesPanel";
import {
  mockLevelPartAnswer,
  mockLevelPartResponses,
  mockLevelTextureReport,
  mockSelectedLevelDescription,
} from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { EMPTY_LEVEL_RESIDENT } from "@/core/level/lib/render/level-resident";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService } from "@/core/level/services/level-load/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockSurfaceDescriptor } from "@/core/visuals/fixtures/visual.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
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
  arrange?: (container: Container) => void
): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
    ...mockLevelPartResponses({ surfaces: TABLE }),
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
  ]);
  const service: LevelLoadService = container.get(LevelLoadService);

  // What a level's textures came to, which no test here streams for itself: given one, the panel reads it.
  if (textures) {
    mockCommandAnswer(
      container.get(LevelRenderService).resident,
      AsyncState.ready({ ...EMPTY_LEVEL_RESIDENT, textures })
    );
  }

  arrange?.(container);

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
  await userEvent.click(view.getByRole("textbox", { name: "Filter surfaces" }));
  await userEvent.paste(filter);
  await userEvent.click(await view.findByText(label));

  return view.findByTestId("level-surface-details");
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
      LevelLookService,
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

    await userEvent.click(view.getByRole("textbox", { name: "Filter surfaces" }));

    await userEvent.paste("decal");

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
    // Nothing is resident in this test, so the renderer has counted nothing yet, which the entry says.
    expect(
      within(details).getByText("Not read yet: the renderer says how much it draws once the level is resident.")
    ).toBeInTheDocument();
  });

  // The renderer counts what every entry draws once the level is resident; the panel shows the chosen entry's count.
  it("shows what the viewport counted the chosen entry drawing", async () => {
    const view: RenderResult = await renderPanel(undefined, (container: Container) =>
      mockCommandAnswer(
        container.get(LevelRenderService).resident,
        AsyncState.ready({
          ...EMPTY_LEVEL_RESIDENT,
          surfaces: new Map(
            [0, 1, 2, 3].map((shaderId) => [shaderId, { drawables: 2, narrowest: null, span: null, triangles: 70 }])
          ),
        })
      )
    );

    const details: HTMLElement = await choose(view, "poteki", "2 · decal\\decal_poteki");

    expect(within(details).getByText("2 drawables · 70 triangles · 35.0 each")).toBeInTheDocument();
  });

  // A click on a surface in the viewport is the other way of choosing its entry, and a filter must not hide it.
  it("chooses the entry a surface clicked in the viewport draws with, clearing a filter that hides it", async () => {
    let viewport: Nullable<LevelViewportService> = null;
    const view: RenderResult = await renderPanel(undefined, (container: Container) => {
      viewport = container.get(LevelViewportService);
    });
    const filter: HTMLElement = view.getByRole("textbox", { name: "Filter surfaces" });

    await userEvent.click(filter);

    await userEvent.paste("default");
    act(() =>
      (viewport as unknown as LevelViewportService).notePicked({
        isImpostor: false,
        kind: ELevelPick.SURFACE,
        mesh: null,
        place: null,
        point: { x: 0, y: 0, z: 0 },
        sector: 3,
        shaderId: 3,
      })
    );

    expect(await view.findByTestId("level-surface-details")).toHaveTextContent(
      "3 · effects\\wallmarkmult · decal\\decal_rza_a"
    );
    expect(filter).toHaveValue("");

    // The tree follows the viewport's selection, so one cleared there chooses nothing here.
    act(() => (viewport as unknown as LevelViewportService).notePicked(null));

    await waitFor(() => expect(view.queryByTestId("level-surface-details")).not.toBeInTheDocument());
  });

  // A table read anew may number its entries afresh: the one chosen is found again by its shader and textures.
  it("keeps the entry chosen across its table read anew, and chooses nothing once it is gone", async () => {
    let container: Nullable<Container> = null;
    const view: RenderResult = await renderPanel(undefined, (it: Container) => {
      container = it;
    });

    async function reread(revision: number, table: Array<XraySurfaceDescriptor>): Promise<void> {
      const held: Container = container as unknown as Container;
      const load: LevelLoadService = held.get(LevelLoadService);

      mockIpcCommands({ [ELevelsCommand.DESCRIBE_SURFACES]: mockLevelPartAnswer(table, revision) });
      act(() =>
        held.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
          revision,
          root: { kind: "kept" },
          sessionId: load.level.value?.selected.sessionId as string,
          staleParts: [LEVEL_SURFACES_PART],
          isViewRenewed: false,
        })
      );
      await waitFor(() => expect(load.surfaces.snapshot?.revision).toBe(revision));
    }

    await choose(view, "decal", "2 · decal\\decal_poteki");
    await reread(1, [TABLE[0], TABLE[1], TABLE[3], TABLE[2]]);

    expect(await view.findByTestId("level-surface-details")).toHaveTextContent(
      "3 · effects\\wallmarkmult · decal\\decal_poteki"
    );
    expect(view.getByRole("textbox", { name: "Filter surfaces" })).toHaveValue("decal");

    await reread(2, [TABLE[0], TABLE[1], TABLE[3]]);

    await waitFor(() => expect(view.queryByTestId("level-surface-details")).not.toBeInTheDocument());
  });
});
