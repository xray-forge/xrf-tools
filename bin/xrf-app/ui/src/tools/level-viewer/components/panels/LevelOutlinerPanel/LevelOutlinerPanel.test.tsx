import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container, EventBus, Injectable } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import {
  ELevelObjectId,
  ELevelObjectKind,
  ELevelPlacementDetails,
  ELevelPlacementFile,
  LEVEL_SPAWN_PART,
  LevelObject,
  LevelObjectsDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import {
  mockLevelObject,
  mockLevelPartAnswer,
  mockLevelVolumeObject,
  mockSelectedLevelDescription,
} from "@/core/level/fixtures/level.mocks";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { ELevelPick, TLevelObjectPick } from "@/core/level/lib/pick/level-pick";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelInspectorPanel } from "@/tools/level-viewer/components/panels/LevelInspectorPanel";
import { LevelViewerObjectsService } from "@/tools/level-viewer/services/level-viewer-objects";

import { LevelOutlinerPanel } from "./LevelOutlinerPanel";

function mockSpawned(index: number, name: string, section: string, overrides: Partial<LevelObject> = {}): LevelObject {
  return mockLevelObject({ id: { index, source: ELevelObjectId.SPAWN }, name, ...overrides }, { section });
}

const OBJECTS: LevelObjectsDescription = {
  objects: [
    mockSpawned(0, "crate_1", "box", { transform: mockVisualTransform({ x: 5, y: 1, z: -8 }) }),
    mockSpawned(1, "crate_2", "box"),
    {
      ...mockSpawned(2, "medkit", "medkit"),
      kind: ELevelObjectKind.ITEMS,
      spawn: { ...(mockSpawned(2, "medkit", "medkit").spawn as NonNullable<LevelObject["spawn"]>), visual: 1 },
    },
    { ...mockLevelVolumeObject(3, 4), name: "fence" },
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

/** Lets the listing, started by the open, land. */
async function settle(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function toPick(object: LevelObject): TLevelObjectPick {
  return { kind: ELevelPick.OBJECT, object, point: { x: 0, y: 0, z: 0 }, visual: "" };
}

async function renderPanel(
  arrange?: (container: Container) => void,
  objects: LevelObjectsDescription = OBJECTS
): Promise<{ container: Container; view: RenderResult }> {
  const level = mockSelectedLevelDescription({ sectors: [] });
  let opened: number = 0;

  mockIpcCommands({
    [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: mockSessionResponse({
      customData: "[logic]\nactive = ph_idle",
      gameVertexId: 12,
      id: 400,
      levelVertexId: 3456,
    }),
    // Each opening is a session of its own, as the backend's are.
    [ELevelsCommand.OPEN_LEVEL]: () => mockSessionSnapshot(level, `level-session-${(opened += 1)}`),
    [ELevelsCommand.DESCRIBE_OBJECTS]: mockSessionResponse(objects),
  });

  const container: Container = mockContainer([
    LevelLoadService,
    { token: LevelRenderService, type: "Instance", value: TestLevelRenderService },
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
    LevelViewerObjectsService,
  ]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngineChoice.VANILLA,
  });
  await settle();
  arrange?.(container);

  return {
    container,
    view: renderWithProviders(
      <>
        <LevelOutlinerPanel />
        <LevelInspectorPanel />
      </>,
      { container }
    ),
  };
}

/** Walks the keyboard from the first group down to the first object of its first kind's first section. */
async function chooseFirstCrate(view: RenderResult): Promise<void> {
  await userEvent.click(await view.findByText("Spawned"));
  await userEvent.keyboard("{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}{ArrowRight}{ArrowDown}");
}

describe("LevelOutlinerPanel", () => {
  beforeEach(() => {
    sent.length = 0;
  });

  it("lists each family of kinds the level holds, counting what it holds", async () => {
    const { view } = await renderPanel();
    const tree: HTMLElement = await view.findByRole("tree", { name: "Level objects" });
    const rows: Array<HTMLElement> = within(tree).getAllByRole("treeitem");

    expect(rows.map((it: HTMLElement) => it.textContent)).toEqual(["Spawned3", "Logic volumes1"]);
  });

  // One state in two places: the eye switches the same option the toolbar's `Spawn` group does.
  it("hides and shows a kind from the eye on its row", async () => {
    const { container, view } = await renderPanel();

    await userEvent.click(await view.findByText("Spawned"));
    await userEvent.keyboard("{ArrowRight}");

    const eye: HTMLElement = await view.findByRole("button", { name: "Show props" });

    expect(eye).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(eye);

    expect(container.get(LevelViewService).options.isSpawnedProps).toBe(false);
    expect(await view.findByRole("button", { name: "Show props" })).toHaveAttribute("aria-pressed", "false");
  });

  // A volume draws nothing of its own: its row has a helper switch and no eye.
  it("switches a kind's helpers, which wait on helpers being drawn at all", async () => {
    const { container, view } = await renderPanel();
    const viewService: LevelViewService = container.get(LevelViewService);

    await userEvent.click(await view.findByText("Logic volumes"));
    await userEvent.keyboard("{ArrowRight}");

    expect(view.queryByRole("button", { name: "Show restrictors" })).not.toBeInTheDocument();
    expect(await view.findByRole("button", { name: "Helpers of restrictors" })).toBeDisabled();

    act(() => viewService.toggleHelpers());

    const helper: HTMLElement = await view.findByRole("button", { name: "Helpers of restrictors" });

    expect(helper).toHaveAttribute("aria-pressed", "true");
    await userEvent.click(helper);
    expect(viewService.helpers.kinds).toEqual({ [ELevelObjectKind.RESTRICTORS]: false });
  });

  it("opens what a filter matches by text or kind, and counts only that", async () => {
    const { view } = await renderPanel();

    await userEvent.type(await view.findByRole("textbox", { name: "Filter level objects" }), "crate_2");

    await waitFor(() =>
      expect(
        within(view.getByRole("tree"))
          .getAllByRole("treeitem")
          .map((it) => it.textContent)
      ).toEqual(["Spawned1", "Props1", "box1", "crate_2"])
    );

    await userEvent.clear(view.getByRole("textbox", { name: "Filter level objects" }));
    await userEvent.click(view.getByRole("button", { name: "Restrictors" }));

    await waitFor(() =>
      expect(
        within(view.getByRole("tree"))
          .getAllByRole("treeitem")
          .map((it) => it.textContent)
      ).toEqual(["Logic volumes1", "Restrictors1", "fence"])
    );
  });

  it("says what a chosen object is in the Inspector, what the backend knows of it, and goes to it framed", async () => {
    const { view } = await renderPanel();

    await chooseFirstCrate(view);

    const details: HTMLElement = await view.findByTestId("level-object-details");

    expect(details).toHaveTextContent("crate_1");
    expect(details).toHaveTextContent("physics\\box");
    expect(details).toHaveTextContent("x 5.0 y 1.0 z 8.0");
    expect(await within(details).findByText(/active = ph_idle/)).toBeInTheDocument();
    expect(details).toHaveTextContent("3456");

    await userEvent.click(view.getByRole("treeitem", { name: "crate_1" }));
    await userEvent.keyboard("{Enter}");
    await waitFor(() => expect(sent).toHaveLength(1));

    // Framed along the view the camera has before it reports: level with it, back along `-z` from it.
    expect(sent[0]).toMatchObject({ heading: 0, pitch: expect.closeTo(-20), x: expect.closeTo(5) });
    expect(sent[0]?.z).toBeLessThan(8);
  });

  it("says what a placement's file and a light's bundle record of it", async () => {
    const { container, view } = await renderPanel();
    const sound: LevelObject = mockLevelObject({
      id: { file: ELevelPlacementFile.SOUNDS, index: 0, source: ELevelObjectId.PLACEMENT },
      kind: ELevelObjectKind.STATIC_SOUNDS,
      name: "ambient\\wind",
      placement: {
        details: {
          active: [0, 0],
          frequency: 1,
          kind: ELevelPlacementDetails.SOUND,
          pause: [1500, 4000],
          play: [0, 0],
          sound: "ambient\\wind",
          volume: 0.8,
        },
        shapes: [],
      },
      spawn: null,
    });

    act(() => container.get(LevelViewportService).notePicked(toPick(sound)));

    const details: HTMLElement = await view.findByTestId("level-object-details");

    expect(details).toHaveTextContent("Static sounds");
    expect(details).toHaveTextContent("All day");
    expect(details).toHaveTextContent("1.5 s to 4 s");

    act(() =>
      container.get(LevelViewportService).notePicked(
        toPick(
          mockLevelObject({
            id: { index: 3, source: ELevelObjectId.LIGHT },
            kind: ELevelObjectKind.LIGHTS,
            light: { color: [1, 0.5, 0.25], isSpot: true, range: 12 },
            name: "light 3",
            spawn: null,
          })
        )
      )
    );

    await waitFor(() =>
      expect(view.getByTestId("level-object-details")).toHaveTextContent("Point, authored as a spot")
    );
    expect(view.getByTestId("level-object-details")).toHaveTextContent("12 m");
  });

  it("says what a volume restricts and the shapes it covers", async () => {
    const { container, view } = await renderPanel();

    act(() => container.get(LevelViewportService).notePicked(toPick(OBJECTS.objects[3] as LevelObject)));

    const details: HTMLElement = await view.findByTestId("level-object-details");

    expect(details).toHaveTextContent("No one by itself");
    expect(details).toHaveTextContent("Sphere, 4 m radius");
  });

  // What a click in the viewport picked is chosen here as a click on its row would choose it, however deep it lies.
  it("chooses an object clicked in the viewport, opening what stands above it", async () => {
    const { container, view } = await renderPanel();

    await view.findByRole("tree", { name: "Level objects" });
    act(() => container.get(LevelViewportService).notePicked(toPick(OBJECTS.objects[2] as LevelObject)));

    expect(await view.findByTestId("level-object-details")).toHaveTextContent("medkit");
    expect(view.getByRole("treeitem", { name: "medkit", selected: true })).toBeInTheDocument();
  });

  // A panel opened after the pick mounts before the listing reaches it; that arrival must not forget the object as a
  // new level's would.
  it("chooses an object picked before the panel mounted, once the listing reaches it", async () => {
    const { view } = await renderPanel((container: Container) =>
      container.get(LevelViewportService).notePicked(toPick(OBJECTS.objects[1] as LevelObject))
    );

    expect(await view.findByTestId("level-object-details")).toHaveTextContent("crate_2");
    expect(view.getByRole("treeitem", { name: "crate_2", selected: true })).toBeInTheDocument();
  });

  // The tree follows the viewport's selection, so one cleared there or moved to a surface chooses nothing here.
  it("lets the chosen object go once the viewport's selection does", async () => {
    const { container, view } = await renderPanel();
    const viewportService: LevelViewportService = container.get(LevelViewportService);

    await view.findByRole("tree", { name: "Level objects" });
    act(() => viewportService.notePicked(toPick(OBJECTS.objects[2] as LevelObject)));

    expect(await view.findByTestId("level-object-details")).toHaveTextContent("medkit");

    act(() => viewportService.notePicked(null));

    await waitFor(() => expect(view.queryByTestId("level-object-details")).not.toBeInTheDocument());
    expect(view.queryByRole("treeitem", { selected: true })).not.toBeInTheDocument();
  });

  // The Inspector is headed with its stripe button's name as every other side panel is, chosen object or none.
  it("heads the Inspector with its name, whatever it shows", async () => {
    const { container, view } = await renderPanel();

    function inspector(): HTMLElement {
      return view.getByTestId("level-inspector-panel");
    }

    expect(within(inspector()).getByText("Inspector")).toBeInTheDocument();

    act(() => container.get(LevelViewportService).notePicked(toPick(OBJECTS.objects[2] as LevelObject)));

    expect(await within(inspector()).findByTestId("level-object-details")).toHaveTextContent("medkit");
    expect(within(inspector()).getByText("Inspector")).toBeInTheDocument();
  });

  it("adds a Ctrl-clicked row to the selection and selects a Shift-clicked run", async () => {
    const { container, view } = await renderPanel();
    const viewportService: LevelViewportService = container.get(LevelViewportService);
    // One session, so a key held down is still held at the click after it.
    const user = userEvent.setup();

    await chooseFirstCrate(view);
    await user.keyboard("{Control>}");
    await user.click(view.getByRole("treeitem", { name: "crate_2" }));
    await user.keyboard("{/Control}");

    expect(viewportService.selected.map((it: LevelObject) => it.name)).toEqual(["crate_1", "crate_2"]);
    expect(view.getByRole("treeitem", { name: "crate_1", selected: true })).toBeInTheDocument();
    expect(await view.findByTestId("level-object-details")).toHaveTextContent("1 more selected");

    await user.click(view.getByRole("treeitem", { name: "crate_1" }));
    await user.keyboard("{Shift>}");
    await user.click(view.getByRole("treeitem", { name: "crate_2" }));
    await user.keyboard("{/Shift}");

    expect(viewportService.selected.map((it: LevelObject) => it.name)).toEqual(["crate_1", "crate_2"]);
    expect(viewportService.picked).toMatchObject({ object: { name: "crate_2" } });
  });

  it("hides one object from the eye on its row", async () => {
    const { container, view } = await renderPanel();

    await chooseFirstCrate(view);
    await userEvent.click(view.getByRole("button", { name: "Hide crate_1" }));

    expect(container.get(LevelViewportService).hidden.map((it: LevelObject) => it.name)).toEqual(["crate_1"]);
    expect(view.getByRole("button", { name: "Hide crate_1" })).toHaveAttribute("aria-pressed", "false");
  });

  // The viewport marks what is picked, so an object chosen in the tree is picked as a click on it is.
  it("picks an object chosen in the tree, which the viewport then marks", async () => {
    const { container, view } = await renderPanel();

    await chooseFirstCrate(view);

    expect(await view.findByTestId("level-object-details")).toBeInTheDocument();
    expect(container.get(LevelViewportService).picked).toMatchObject({ kind: ELevelPick.OBJECT });
  });

  // The selection itself is let go by the render service as a level opens; the tree lets its own cursor go.
  it("forgets the row chosen when the level opens again", async () => {
    const { container, view } = await renderPanel();
    const service: LevelLoadService = container.get(LevelLoadService);

    await chooseFirstCrate(view);

    expect(await view.findByTestId("level-object-details")).toBeInTheDocument();

    // The level opening again redraws the panel as each of its steps lands.
    await act(async () => {
      await service.load({
        source: { kind: "asset", logicalPath: "levels\\zaton" },
        roots: mockSelectedLevelDescription().roots,
        isDltx: false,
        engine: EXrayEngineChoice.VANILLA,
      });
      await settle();
    });

    const tree: HTMLElement = await view.findByRole("tree", { name: "Level objects" });

    expect(tree).not.toHaveAttribute("aria-activedescendant");
  });

  // A listing read anew may number its objects afresh: the core finds what is picked again by its name, and the tree
  // chooses it where it stands now, its filter kept.
  it("keeps the object chosen across the listing read anew", async () => {
    const { container, view } = await renderPanel();
    const load: LevelLoadService = container.get(LevelLoadService);
    const viewportService: LevelViewportService = container.get(LevelViewportService);
    const filter: HTMLElement = await view.findByRole("textbox", { name: "Filter level objects" });
    const [first, second, medkit, fence] = OBJECTS.objects as Array<LevelObject>;
    const renumbered: LevelObjectsDescription = {
      ...OBJECTS,
      objects: [medkit, second, first, fence].map((it, index) => ({
        ...(it as LevelObject),
        id: { index, source: ELevelObjectId.SPAWN },
      })),
    };

    await userEvent.type(filter, "crate");
    await userEvent.click(await view.findByText("crate_1"));
    expect(await view.findByTestId("level-object-details")).toHaveTextContent("crate_1");

    mockIpcCommands({ [ELevelsCommand.DESCRIBE_OBJECTS]: mockLevelPartAnswer(renumbered, 1) });
    act(() =>
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
        revision: 1,
        root: { kind: "kept" },
        sessionId: load.level.value?.selected.sessionId as string,
        staleParts: [LEVEL_SPAWN_PART],
        isViewRenewed: false,
      })
    );
    await waitFor(() => expect(load.objectsRevision).toBe(1));
    act(() => viewportService.notePickRelocated(toPick(renumbered.objects[2] as LevelObject)));

    expect(view.getByRole("treeitem", { name: "crate_1", selected: true })).toBeInTheDocument();
    expect(filter).toHaveValue("crate");
  });

  // Before the level's objects are listed the report is the empty one, which reads as a spawn placing nothing.
  it("says the spawn is being read until its objects are listed, then that it places nothing", async () => {
    const { container, view } = await renderPanel(undefined, { objects: [], visuals: [] });
    const service: LevelLoadService = container.get(LevelLoadService);

    expect(view.getByText("The level's spawn places nothing.")).toBeInTheDocument();

    // Let go and not listed yet: what the report says before a spawn's first listing.
    act(() => service.levelObjects.detach());

    expect(view.getByText("Reading the level's spawn.")).toBeInTheDocument();
  });
});
