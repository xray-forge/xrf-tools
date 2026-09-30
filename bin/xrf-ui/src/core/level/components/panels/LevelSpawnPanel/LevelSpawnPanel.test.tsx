import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container, Injectable } from "@wirestate/core";

import { ELevelSpawnCategory, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import {
  LevelLoadService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
  LevelWeatherService,
} from "@/core/level/services";
import { setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import { mockLevelSpawnModel, mockLevelSpawnObject, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelSpawnPanel } from "./LevelSpawnPanel";

/** Two crates among the props and a medkit among the items. */
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

/** Where the panel sent the camera, which the renderer this test starts none of would stand. */
const sent: Array<ILevelGoTo> = [];

@Injectable()
class TestLevelRenderService extends LevelRenderService {
  public override goTo(goTo: ILevelGoTo): void {
    sent.push(goTo);
  }
}

async function renderPanel(): Promise<{ container: Container; view: RenderResult }> {
  const level = mockSelectedLevelDescription({ sectors: [] });

  setMockInvokeResponses({
    ["plugin:levels|describe_spawn_models"]: mockSessionResponse((args?: Record<string, unknown>) => ({
      failures: [],
      hemi: [],
      models: (args?.names as Array<string>).map((name: string) => mockLevelSpawnModel(name).description),
    })),
    ["plugin:levels|describe_spawn_object"]: mockSessionResponse({
      customData: "[logic]\nactive = ph_idle",
      gameVertexId: 12,
      id: 400,
      levelVertexId: 3456,
    }),
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_spawn_objects"]: mockSessionResponse(OBJECTS),
  });
  setMockBulkResponses({
    "levels/read_spawn_model": (args: Record<string, unknown>) => mockLevelSpawnModel(String(args.name)).buffer,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    { token: LevelRenderService, type: "Instance", value: TestLevelRenderService },
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
  ]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngine.VANILLA,
  });
  await service.whenHeldRead();

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

    await userEvent.click(await view.findByRole("button", { name: "Hide props" }));

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

    // Framed along the view the camera has before it reports: level with it, back along `-z` from it.
    expect(sent).toHaveLength(1);
    expect(sent[0]).toMatchObject({ heading: 0, pitch: expect.closeTo(-20), x: expect.closeTo(5) });
    expect(sent[0].z).toBeLessThan(8);
  });

  // Another opening numbers its objects afresh, so object 0 of the last level is not object 0 of this one.
  it("forgets the chosen object when the level opens again", async () => {
    const { container, view } = await renderPanel();
    const service: LevelLoadService = container.get(LevelLoadService);

    await userEvent.click(await view.findByText("Props"));
    await userEvent.keyboard("{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}");

    expect(await view.findByTestId("level-spawn-details")).toBeInTheDocument();

    await service.load({
      source: { kind: "asset", logicalPath: "levels\\zaton" },
      roots: mockSelectedLevelDescription().roots,
      isDltx: false,
      engine: EXrayEngine.VANILLA,
    });
    await service.whenHeldRead();

    await waitFor(() => expect(view.queryByTestId("level-spawn-details")).not.toBeInTheDocument());
    expect(await view.findByRole("tree", { name: "Spawned objects" })).toBeInTheDocument();
  });
});
