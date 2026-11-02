import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import {
  ELevelObjectId,
  LEVEL_GEOMETRY_PART,
  LEVEL_SPAWN_PART,
  LEVEL_SURFACES_PART,
  LevelObject,
} from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderLevelHit, ERenderSelectionTarget, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";

import { LEVEL_VIEWPORT, mockAttachedLevelRender } from "@/core/level/fixtures/level-render";
import { mockLevelObject, mockLevelPartAnswer, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelViewService } from "@/core/level/services/level-view";
import { ELevelSelectionMode, LevelViewportService } from "@/core/level/services/level-viewport";
import { clearLocalStorage } from "@/core/storage";
import { mockSurfaceDescriptor, mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";
describe("LevelRenderPicks", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("names what a pick hit in the level's own coordinates", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const hit: RenderLevelHit = {
      depth: 3,
      isImpostor: false,
      kind: ERenderLevelHit.SURFACE,
      mesh: 2,
      place: 7,
      point: [1, 2, 3],
      sector: 4,
      shaderId: 11,
    };

    mockIpcCommands({ [ERenderCommand.PICK]: hit });
    await service.picks.pick({ isThrough: true, point: { x: 10, y: 20 } });

    expect(listMockInvokeCalls(ERenderCommand.PICK).at(-1)).toEqual({
      viewport: LEVEL_VIEWPORT,
      request: { x: 10, y: 20, isThrough: true },
    });
    expect(container.get(LevelViewportService).picked).toEqual({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 7,
      point: { x: 1, y: 2, z: -3 },
      sector: 4,
      shaderId: 11,
    });
  });

  // A pick a frame away answers the pointer; points arriving meanwhile wait for it, the latest of them alone picked next.
  it("hovers what the pointer rests on with one pick in flight, picking the latest point next", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const answers: Array<(hit: RenderLevelHit) => void> = [];

    mockIpcCommands({
      [ERenderCommand.PICK]: () =>
        new Promise<RenderLevelHit>((resolve: (hit: RenderLevelHit) => void) => answers.push(resolve)),
    });

    const first = service.picks.hover({ x: 1, y: 1 });
    const second = service.picks.hover({ x: 2, y: 2 });
    const third = service.picks.hover({ x: 3, y: 3 });

    await waitFor(() => expect(answers).toHaveLength(1));
    answers[0]?.({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 0, point: [0, 0, 0] });
    await waitFor(() => expect(answers).toHaveLength(2));
    answers[1]?.({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 0, point: [0, 0, 0] });
    await Promise.allSettled([first, second, third]);

    expect(listMockInvokeCalls(ERenderCommand.PICK).map((call) => call.request)).toEqual([
      { isThrough: false, x: 1, y: 1 },
      { isThrough: false, x: 3, y: 3 },
    ]);

    await service.picks.hover(null);

    expect(container.get(LevelViewportService).hovered).toBeNull();
  });

  it("focuses the view's lens where a click lands while the focus is picked, selecting nothing", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 12.345, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] },
    });
    viewport.setFocusPicking(true);
    await service.picks.focusAt({ isThrough: false, point: { x: 10, y: 20 } });

    expect(viewport.isFocusPicking).toBe(false);
    expect(viewport.picked).toBeNull();
    expect(container.get(LevelViewService).features.camera).toEqual({ focusDistance: 12.35, isDefocused: true });
  });

  it("keeps what is picked, selected and hidden across the spawn read anew, found by name, and lets each go once gone", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);
    const sessionId: string = load.level.value?.selected.sessionId as string;
    const lamp: LevelObject = mockLevelObject({ id: { index: 0, source: ELevelObjectId.SPAWN }, name: "lamp" });
    const crate: LevelObject = mockLevelObject({
      id: { index: 1, source: ELevelObjectId.SPAWN },
      name: "crate",
      transform: mockVisualTransform({ x: 1, y: 2, z: 3 }),
    });
    const fence: LevelObject = { ...mockLevelVolumeObject(2), name: "fence" };

    async function relist(revision: number, objects: Array<LevelObject>): Promise<void> {
      mockIpcCommands({
        [ELevelsCommand.DESCRIBE_OBJECTS]: mockLevelPartAnswer({ objects, visuals: ["lamp"] }, revision),
        [ERenderCommand.PICK]: { depth: 0, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] },
      });
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
        revision,
        root: { kind: "kept" },
        sessionId,
        staleParts: [LEVEL_SPAWN_PART],
        isViewRenewed: false,
      });
      await waitFor(() => expect(load.objectsRevision).toBe(revision));
    }

    await relist(1, [lamp, crate, fence]);
    await service.picks.pick({ isThrough: false, point: { x: 10, y: 20 } });
    expect(viewport.picked).toMatchObject({ kind: ELevelPick.OBJECT, object: crate });
    viewport.toggleHidden(fence);

    await relist(2, [
      { ...crate, id: { index: 0, source: ELevelObjectId.SPAWN } },
      { ...lamp, id: { index: 1, source: ELevelObjectId.SPAWN } },
      { ...fence, id: { index: 5, source: ELevelObjectId.SPAWN } },
    ]);
    expect(viewport.picked).toMatchObject({ kind: ELevelPick.OBJECT, object: { id: { index: 0 }, name: "crate" } });
    expect(viewport.selected).toMatchObject([{ id: { index: 0 }, name: "crate" }]);
    expect(viewport.hidden).toMatchObject([{ id: { index: 5 }, name: "fence" }]);
    expect(viewport.isPickRelocated).toBe(true);
    expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection?.target).toEqual({
      kind: ERenderSelectionTarget.SPAWN,
      object: 0,
    });

    await relist(3, [lamp]);
    expect(viewport.picked).toBeNull();
    expect(viewport.selected).toEqual([]);
    expect(viewport.hidden).toEqual([]);
  });

  it("adds a Ctrl-clicked object to the selection, and takes it out on a second", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const lamp: LevelObject = mockLevelObject({ id: { index: 0, source: ELevelObjectId.SPAWN }, name: "lamp" });
    const crate: LevelObject = mockLevelObject({ id: { index: 1, source: ELevelObjectId.SPAWN }, name: "crate" });

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_OBJECTS]: mockLevelPartAnswer({ objects: [lamp, crate], visuals: ["lamp"] }, 1),
    });
    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: "kept" },
      sessionId: container.get(LevelLoadService).level.value?.selected.sessionId as string,
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    await waitFor(() => expect(container.get(LevelLoadService).objectsRevision).toBe(1));

    mockIpcCommands({ [ERenderCommand.PICK]: { depth: 0, kind: ERenderLevelHit.SPAWN, object: 0, point: [0, 0, 0] } });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 1 } });
    mockIpcCommands({ [ERenderCommand.PICK]: { depth: 0, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] } });
    await service.picks.pick({ isThrough: false, mode: ELevelSelectionMode.TOGGLE, point: { x: 2, y: 2 } });

    expect(viewport.selected.map((object: LevelObject) => object.name)).toEqual(["lamp", "crate"]);
    expect(viewport.picked).toMatchObject({ object: { name: "crate" } });

    await service.picks.pick({ isThrough: false, mode: ELevelSelectionMode.TOGGLE, point: { x: 2, y: 2 } });

    expect(viewport.selected.map((object: LevelObject) => object.name)).toEqual(["lamp"]);
    expect(viewport.picked).toBeNull();
  });

  // A table read anew may number its entries afresh; the surface keeps the place it was picked at.
  it("keeps a surface picked across its level streamed afresh, its entry found again by shader and textures", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);
    const sessionId: string = load.level.value?.selected.sessionId as string;
    const wall = mockSurfaceDescriptor({ shader: "default", textures: ["wall"] });
    const floor = mockSurfaceDescriptor({ shader: "default", textures: ["floor"] });

    async function reread(revision: number, surfaces: Array<XraySurfaceDescriptor>): Promise<void> {
      mockIpcCommands({
        [ELevelsCommand.DESCRIBE_SURFACES]: mockLevelPartAnswer(surfaces, revision),
        [ERenderCommand.PICK]: {
          depth: 0,
          isImpostor: false,
          kind: ERenderLevelHit.SURFACE,
          mesh: null,
          place: null,
          point: [0, 0, 0],
          sector: 4,
          shaderId: 0,
        },
      });
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
        revision,
        root: { kind: "kept" },
        sessionId,
        staleParts: [LEVEL_GEOMETRY_PART, LEVEL_SURFACES_PART],
        isViewRenewed: true,
      });
      await waitFor(() => expect(load.surfaces.snapshot?.revision).toBe(revision));
    }

    await reread(1, [wall, floor]);
    await service.picks.pick({ isThrough: false, point: { x: 10, y: 20 } });
    await reread(2, [floor, wall]);

    expect(viewport.picked).toMatchObject({ kind: ELevelPick.SURFACE, sector: 4, shaderId: 1 });

    await reread(3, [floor]);

    expect(viewport.picked).toBeNull();
  });
});
