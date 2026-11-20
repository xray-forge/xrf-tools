import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Binding, Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { listMockInvokeCalls, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import {
  ESceneProblemKind,
  ESceneToolKind,
  SceneObjectsDescription,
  SceneSpawnField,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelEditorInspectorPanel } from "@/tools/level-editor/components/panels/LevelEditorInspectorPanel";
import {
  mockSceneObject,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockScenePartResponses,
  mockSceneSpawn,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorObjectsService } from "@/tools/level-editor/services/level-editor-objects";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorOutlinerPanel } from "./LevelEditorOutlinerPanel";

const OUTLINER_BINDINGS: Array<Binding> = [
  LevelEditorSceneService,
  LevelEditorSelectionService,
  LevelEditorHelpersService,
  LevelViewService,
  LevelViewportService,
  LevelEditorRenderService,
  LevelEditorObjectsService,
];

function mockGroupedObjects(): SceneObjectsDescription {
  const [sceneObjects, spawns] = mockSceneObjectsDescription().tools;

  return {
    problems: [],
    tools: [
      mockSceneObjectTool(ESceneToolKind.SCENE_OBJECT, [
        ...(sceneObjects?.objects ?? []),
        mockSceneObject({ group: "lab_props", id: { index: 2, tool: ESceneToolKind.SCENE_OBJECT }, name: "chair" }),
      ]),
      ...(spawns ? [spawns] : []),
    ],
  };
}

async function renderPanels(
  objects: SceneObjectsDescription = mockGroupedObjects(),
  container: Container = mockContainer(OUTLINER_BINDINGS)
): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: () =>
      mockSessionSnapshot(
        mockSelectedSceneDescription({
          problems: [
            {
              file: "spawn.part",
              kind: ESceneProblemKind.WRONG_VERSION,
              message: "Part holds tool version 1",
              tool: ESceneToolKind.SPAWN,
            },
          ],
        }),
        "scene-session"
      ),
    ...mockScenePartResponses({ objects }),
  });

  await container.get(LevelEditorSceneService).open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: { path: "C:\\labx8.level" },
  });
  await flush();

  return renderWithProviders(
    <>
      <LevelEditorOutlinerPanel />
      <LevelEditorInspectorPanel />
    </>,
    { container, route: "/level-editor" }
  );
}

function listRows(view: RenderResult): Array<string | null> {
  return within(view.getByRole("tree", { name: "Scene outline" }))
    .getAllByRole("treeitem")
    .map((it: HTMLElement) => it.textContent);
}

describe("LevelEditorOutlinerPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("lists every tool in the SDK's order with what it holds, a binary tool counted", async () => {
    const view: RenderResult = await renderPanels();

    expect(listRows(view)).toEqual(["Scene objects3", "Spawn points3", "AI map12,480 nodes"]);
  });

  it("marks a tool its scene's problems name", async () => {
    const view: RenderResult = await renderPanels();

    expect(view.getByLabelText("1 problem, listed in Problems")).toBeInTheDocument();
  });

  it("says why the objects could not be read, and lists nothing", async () => {
    const container: Container = mockContainer(OUTLINER_BINDINGS);

    mockIpcCommands({
      [ELevelsCommand.OPEN_SCENE]: () => mockSessionSnapshot(mockSelectedSceneDescription(), "scene-session"),
      ...mockScenePartResponses(),
      [ELevelsCommand.DESCRIBE_SCENE_OBJECTS]: () => {
        throw new Error("scene_object.part does not read");
      },
    });

    await container.get(LevelEditorSceneService).open({
      engine: EXrayEngineChoice.AUTO,
      gameRoots: null,
      isDltx: false,
      objectsRoots: null,
      source: { path: "C:\\labx8.level" },
    });
    await flush();

    const { getByText } = renderWithProviders(<LevelEditorOutlinerPanel />, { container, route: "/level-editor" });

    expect(getByText("The scene's objects could not be read: scene_object.part does not read")).toBeInTheDocument();
  });

  it("opens what a filter matches by name or section, counting only that, and names a member's group", async () => {
    const view: RenderResult = await renderPanels();

    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "stalker");

    await waitFor(() => expect(listRows(view)).toEqual(["Spawn points2", "stalker_1", "stalker_2"]));

    await userEvent.clear(view.getByRole("textbox", { name: "Filter scene objects" }));
    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "chair");

    await waitFor(() => expect(listRows(view)).toEqual(["Scene objects1", "chairlab_props"]));
  });

  it("shows the object chosen in the Inspector, with what its kind keeps", async () => {
    const view: RenderResult = await renderPanels();

    expect(view.getByText("Nothing selected. Pick an object in the Outliner.")).toBeInTheDocument();

    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "box_2");
    await userEvent.click(await view.findByText("box_2"));

    const inspector: HTMLElement = view.getByTestId("level-editor-inspector-panel");

    expect(within(inspector).getByText("Scene objects")).toBeInTheDocument();
    expect(within(inspector).getByText("props\\crate")).toBeInTheDocument();
    expect(within(inspector).getByText("Not found in the objects roots")).toBeInTheDocument();
    expect(view.getByRole("treeitem", { name: /box_2/ })).toHaveAttribute("aria-current", "true");
  });

  it("opens the tool of an object selected elsewhere and chooses its row, clearing the filter", async () => {
    const container: Container = mockContainer(OUTLINER_BINDINGS);
    const view: RenderResult = await renderPanels(mockGroupedObjects(), container);

    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "chair");
    await waitFor(() => expect(listRows(view)).toEqual(["Scene objects1", "chairlab_props"]));

    act(() => container.get(LevelEditorSelectionService).select({ index: 2, tool: ESceneToolKind.SPAWN }));

    await waitFor(() =>
      expect(view.getByRole("treeitem", { name: /stalker_2/ })).toHaveAttribute("aria-selected", "true")
    );
    expect(view.getByRole("textbox", { name: "Filter scene objects" })).toHaveValue("");
    expect(view.getByRole("treeitem", { name: /stalker_2/ })).toHaveAttribute("aria-current", "true");
    expect(view.getByRole("treeitem", { name: /Spawn points/ })).toHaveAttribute("aria-expanded", "true");

    act(() => container.get(LevelEditorSelectionService).select(null));

    await waitFor(() =>
      expect(view.getByRole("treeitem", { name: /stalker_2/ })).toHaveAttribute("aria-selected", "false")
    );
  });

  it("selects and frames an object whose row is activated", async () => {
    const container: Container = mockContainer(OUTLINER_BINDINGS);
    const view: RenderResult = await renderPanels(mockGroupedObjects(), container);

    mockIpcCommands({
      [ELevelsCommand.LOCATE_SCENE_OBJECT]: (args: { sessionId: string }) =>
        mockSessionSnapshot({ center: { x: 1, y: 2, z: 3 }, radius: 1 }, args.sessionId),
    });
    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "box_2");
    await userEvent.dblClick(await view.findByText("box_2"));

    await waitFor(() =>
      expect(listMockInvokeCalls(ELevelsCommand.LOCATE_SCENE_OBJECT)).toEqual([
        { object: { index: 1, tool: ESceneToolKind.SCENE_OBJECT }, sessionId: "scene-session" },
      ])
    );
    expect(container.get(LevelEditorSelectionService).selectedId).toEqual({
      index: 1,
      tool: ESceneToolKind.SCENE_OBJECT,
    });
  });

  it("hides an object from its row's eye until shown again, and reveals every one hidden", async () => {
    const container: Container = mockContainer(OUTLINER_BINDINGS);
    const view: RenderResult = await renderPanels(mockGroupedObjects(), container);
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "stalker");
    await userEvent.click(await view.findByRole("button", { name: "Hide stalker_1" }));

    expect(selection.hidden).toEqual([{ index: 0, tool: ESceneToolKind.SPAWN }]);
    expect(view.getByRole("button", { name: "Hide stalker_1" })).toHaveAttribute("aria-pressed", "false");
    expect(view.getByText("1 hidden")).toBeInTheDocument();

    await userEvent.click(view.getByRole("button", { name: "Hide stalker_2" }));
    await userEvent.click(view.getByRole("button", { name: "Hide stalker_1" }));

    expect(selection.hidden).toEqual([{ index: 2, tool: ESceneToolKind.SPAWN }]);

    await userEvent.click(view.getByRole("button", { name: "Reveal hidden objects" }));

    expect(selection.hidden).toEqual([]);
    expect(view.queryByText(/hidden$/)).not.toBeInTheDocument();
  });

  // A spawn's packet can run to hundreds of fields.
  it("caps a long list of an object's properties with a count of the rest", async () => {
    const data: Array<SceneSpawnField> = Array.from({ length: 250 }, (_, index: number) => ({
      key: `field_${index}`,
      value: "1",
    }));
    const view: RenderResult = await renderPanels({
      problems: [],
      tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "trader", "trader", data)])],
    });

    await userEvent.type(view.getByRole("textbox", { name: "Filter scene objects" }), "trader");
    await userEvent.click(await view.findByText("trader"));

    expect(view.getByText("Spawn data (250)")).toBeInTheDocument();
    expect(view.getByText(/field_19 = 1\s+and 230 more/)).toBeInTheDocument();
    expect(view.queryByText(/field_20 = 1/)).not.toBeInTheDocument();
  });
});
