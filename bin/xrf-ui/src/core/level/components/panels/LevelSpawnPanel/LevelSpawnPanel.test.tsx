import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container, Injectable } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ELevelSpawnCategory, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { EMPTY_LEVEL_SPAWN_REPORT } from "@/core/level/lib/spawn";
import {
  LevelLoadService,
  LevelLookService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
  LevelWeatherService,
} from "@/core/level/services";
import { mockLevelSpawnObject, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelSpawnPanel } from "./LevelSpawnPanel";

const OBJECTS: LevelSpawnObjectsDescription = {
  objects: [
    mockLevelSpawnObject({
      index: 0,
      name: "crate_1",
      section: "box",
      transform: mockVisualTransform({ x: 5, y: 1, z: -8 }),
    }),
    mockLevelSpawnObject({ index: 1, name: "crate_2", section: "box" }),
    mockLevelSpawnObject({
      category: ELevelSpawnCategory.ITEMS,
      index: 2,
      name: "medkit",
      section: "medkit",
      visual: 1,
    }),
  ],
  visuals: ["physics\\box", "dynamics\\medkit"],
};

const sent: Array<ILevelGoTo> = [];

@Injectable()
class TestLevelRenderService extends LevelRenderService {
  public override goTo(goTo: ILevelGoTo): void {
    sent.push(goTo);
  }

  // The sphere the renderer holds for the crate's model, a metre and a half up from where it stands.
  public override async locateSpawnObject(): Promise<[number, number, number, number]> {
    return [5, 1.5, 8, 0.75];
  }
}

/** Lets the spawn's listing, started by the open, land. */
async function settle(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

async function renderPanel(
  arrange?: (container: Container) => void,
  objects: LevelSpawnObjectsDescription = OBJECTS
): Promise<{ container: Container; view: RenderResult }> {
  const level = mockSelectedLevelDescription({ sectors: [] });

  setMockInvokeResponses({
    ["plugin:levels|describe_spawn_object"]: mockSessionResponse({
      customData: "[logic]\nactive = ph_idle",
      gameVertexId: 12,
      id: 400,
      levelVertexId: 3456,
    }),
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_spawn_objects"]: mockSessionResponse(objects),
  });

  const container: Container = mockContainer([
    LevelLoadService,
    { token: LevelRenderService, type: "Instance", value: TestLevelRenderService },
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
  ]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngine.VANILLA,
  });
  await settle();
  arrange?.(container);

  return { container, view: renderWithProviders(<LevelSpawnPanel />, { container }) };
}

describe("LevelSpawnPanel", () => {
  beforeEach(() => {
    sent.length = 0;
  });

  it("lists each category the spawn places, counting what it holds", async () => {
    const { view } = await renderPanel();
    const tree: HTMLElement = await view.findByRole("tree", { name: "Spawned objects" });
    const rows: Array<HTMLElement> = within(tree).getAllByRole("treeitem");

    expect(rows.map((it: HTMLElement) => it.textContent)).toEqual(["Props2", "Items1"]);
  });

  // One state in two places: the eye switches the same option the toolbar's `Spawn` group does.
  it("hides and shows a category from the eye on its row", async () => {
    const { container, view } = await renderPanel();

    const eye: HTMLElement = await view.findByRole("button", { name: "Show props" });

    expect(eye).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(eye);

    expect(container.get(LevelViewService).options.isSpawnedProps).toBe(false);
    expect(await view.findByRole("button", { name: "Show props" })).toHaveAttribute("aria-pressed", "false");
  });

  it("opens what a filter matches, and counts only that", async () => {
    const { view } = await renderPanel();

    await userEvent.type(await view.findByRole("textbox", { name: "Filter spawned objects" }), "crate_2");

    await waitFor(() =>
      expect(
        within(view.getByRole("tree"))
          .getAllByRole("treeitem")
          .map((it) => it.textContent)
      ).toEqual(["Props1", "box1", "crate_2"])
    );
  });

  it("says what a chosen object is, what the backend knows of it, and goes to it framed", async () => {
    const { view } = await renderPanel();

    await userEvent.click(await view.findByText("Props"));
    await userEvent.keyboard("{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}");

    const details: HTMLElement = await view.findByTestId("level-spawn-details");

    expect(details).toHaveTextContent("crate_1");
    expect(details).toHaveTextContent("physics\\box");
    expect(details).toHaveTextContent("x 5.0 y 1.0 z 8.0");
    expect(await within(details).findByText(/active = ph_idle/)).toBeInTheDocument();
    expect(details).toHaveTextContent("3456");

    await userEvent.keyboard("{Enter}");
    await waitFor(() => expect(sent).toHaveLength(1));

    // Framed along the view the camera has before it reports: level with it, back along `-z` from it.
    expect(sent).toHaveLength(1);
    expect(sent[0]).toMatchObject({ heading: 0, pitch: expect.closeTo(-20), x: expect.closeTo(5) });
    expect(sent[0].z).toBeLessThan(8);
  });

  // What a click in the viewport picked is chosen here as a click on its row would choose it, however deep it lies.
  it("chooses an object clicked in the viewport, opening what stands above it", async () => {
    const { container, view } = await renderPanel();

    await view.findByRole("tree", { name: "Spawned objects" });
    act(() =>
      container.get(LevelViewportService).notePicked({
        kind: ELevelPick.SPAWN,
        object: OBJECTS.objects[2],
        point: { x: 0, y: 0, z: 0 },
        visual: "dynamics\\medkit",
      })
    );

    expect(await view.findByTestId("level-spawn-details")).toHaveTextContent("medkit");
    expect(view.getByRole("treeitem", { name: "medkit", selected: true })).toBeInTheDocument();
  });

  // A panel opened after the pick mounts before the spawn it lists reaches it; that arrival must not forget the object
  // as a new level's would.
  it("chooses an object picked before the panel mounted, once the spawn reaches it", async () => {
    const { view } = await renderPanel((container: Container) =>
      container.get(LevelViewportService).notePicked({
        kind: ELevelPick.SPAWN,
        object: OBJECTS.objects[1],
        point: { x: 0, y: 0, z: 0 },
        visual: OBJECTS.visuals[0],
      })
    );

    expect(await view.findByTestId("level-spawn-details")).toHaveTextContent("crate_2");
    expect(view.getByRole("treeitem", { name: "crate_2", selected: true })).toBeInTheDocument();
  });

  // The tree follows the viewport's selection, so one cleared there or moved to a surface chooses nothing here.
  it("lets the chosen object go once the viewport's selection does", async () => {
    const { container, view } = await renderPanel();
    const viewportService: LevelViewportService = container.get(LevelViewportService);

    await view.findByRole("tree", { name: "Spawned objects" });
    act(() =>
      viewportService.notePicked({
        kind: ELevelPick.SPAWN,
        object: OBJECTS.objects[2],
        point: { x: 0, y: 0, z: 0 },
        visual: "dynamics\\medkit",
      })
    );

    expect(await view.findByTestId("level-spawn-details")).toHaveTextContent("medkit");

    act(() => viewportService.notePicked(null));

    await waitFor(() => expect(view.queryByTestId("level-spawn-details")).not.toBeInTheDocument());
    expect(view.queryByRole("treeitem", { selected: true })).not.toBeInTheDocument();
  });

  // The viewport marks what is picked, so an object chosen in the tree is picked as a click on it is.
  it("picks an object chosen in the tree, which the viewport then marks", async () => {
    const { container, view } = await renderPanel();

    await userEvent.click(await view.findByText("Props"));
    await userEvent.keyboard("{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}");

    expect(await view.findByTestId("level-spawn-details")).toBeInTheDocument();
    expect(container.get(LevelViewportService).picked).toMatchObject({ kind: ELevelPick.SPAWN });
  });

  it("forgets the chosen object when the level opens again", async () => {
    const { container, view } = await renderPanel();
    const service: LevelLoadService = container.get(LevelLoadService);

    await userEvent.click(await view.findByText("Props"));
    await userEvent.keyboard("{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}");

    expect(await view.findByTestId("level-spawn-details")).toBeInTheDocument();

    // The level opening again redraws the panel as each of its steps lands.
    await act(async () => {
      await service.load({
        source: { kind: "asset", logicalPath: "levels\\zaton" },
        roots: mockSelectedLevelDescription().roots,
        isDltx: false,
        engine: EXrayEngine.VANILLA,
      });
      await settle();
    });

    await waitFor(() => expect(view.queryByTestId("level-spawn-details")).not.toBeInTheDocument());
    expect(await view.findByRole("tree", { name: "Spawned objects" })).toBeInTheDocument();
  });

  // Before the spawn's objects are listed the report is the empty one, which reads as a spawn placing nothing.
  it("says the spawn is being read until its objects are listed, then that it places nothing", async () => {
    const { container, view } = await renderPanel(undefined, { objects: [], visuals: [] });
    const service: LevelLoadService = container.get(LevelLoadService);

    expect(view.getByText("The level's spawn places nothing the viewer draws.")).toBeInTheDocument();

    act(() => runInAction(() => (service.spawnReport = EMPTY_LEVEL_SPAWN_REPORT)));

    expect(view.getByText("Reading the level's spawn.")).toBeInTheDocument();
  });
});
