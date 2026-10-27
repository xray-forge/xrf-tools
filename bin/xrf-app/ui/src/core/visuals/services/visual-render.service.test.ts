import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import {
  getMockChannels,
  listMockInvokeCalls,
  MockChannel,
  mockIpcCommands,
  resetMockChannels,
  resetMockInvoke,
} from "@xrf/ipc/testing";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";
import {
  ERenderOverlay,
  ERenderTextureGroup,
  ERenderTextureState,
  ERenderViewportEvent,
} from "@xrf/ipc/types/xrf-renderer";
import { EWorldCameraCommand } from "@xrf/ipc/types/xrf-world";

import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import {
  mockLoadedVisual,
  mockSelectedVisual,
  mockTextureDependency,
  mockVisualModelViews,
} from "@/core/visuals/fixtures/visual.mocks";
import { BIND_POSE, IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { mockContainer } from "@/fixtures/utils/container";

const VIEWPORT: number = 3;
/** The scene the mocked renderer draws every model shown as. */
const SCENE: number = 4;

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function emit(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}

function mockSource(overrides: Partial<IVisualRenderSource> = {}): IVisualRenderSource {
  return makeAutoObservable<IVisualRenderSource>(
    { hasBump: false, hiddenBoneIndices: new Set(), model: null, pose: BIND_POSE, sessionId: null, ...overrides },
    {},
    { deep: false }
  );
}

async function mockAttached(source: IVisualRenderSource): Promise<{
  container: Container;
  service: VisualRenderService;
  viewService: VisualViewService;
}> {
  mockIpcCommands({ [ERenderCommand.ATTACH_VIEWPORT]: VIEWPORT, [EVisualsCommand.SHOW_MODEL]: SCENE });

  const container: Container = mockContainer([
    VisualLoadService,
    VisualViewService,
    VisualRenderService,
    { factory: () => source, token: VISUAL_RENDER_SOURCE },
  ]);
  const service: VisualRenderService = container.get(VisualRenderService);

  service.attach(document.createElement("div"));
  await flush();

  return { container, service, viewService: container.get(VisualViewService) };
}

describe("VisualRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
  });

  it("shows the open model's session at the toolbar's detail, and again as either changes", async () => {
    const source: IVisualRenderSource = mockSource({ model: mockVisualModelViews(), sessionId: "first" });
    const { service, viewService } = await mockAttached(source);

    viewService.setDetail(0.5);
    runInAction(() => (source.sessionId = "second"));
    await flush();

    expect(listMockInvokeCalls(EVisualsCommand.SHOW_MODEL).map(({ detail, sessionId }) => [sessionId, detail])).toEqual(
      [
        ["first", 0],
        ["first", 0.5],
        ["second", 0.5],
      ]
    );

    service.dispose();
  });

  it("stands the model in the source's motion frame, its hidden bones collapsed", async () => {
    const source: IVisualRenderSource = mockSource({ model: mockVisualModelViews(), sessionId: "first" });
    const { service } = await mockAttached(source);

    runInAction(() => {
      source.pose = { frame: 4, motion: "walk" };
      source.hiddenBoneIndices = new Set([2, 3]);
    });
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.POSE_MODEL).map(({ pose }) => pose)).toEqual([
      { ...BIND_POSE, hiddenBones: [] },
      { frame: 4, hiddenBones: [2, 3], motion: "walk" },
    ]);

    service.dispose();
  });

  it("draws the skeleton and the joint marker only while the skeleton is shown", async () => {
    const source: IVisualRenderSource = mockSource({
      highlightedJoint: [0, 1, 0],
      model: mockVisualModelViews({ hasSkeleton: true }),
      sessionId: "first",
    });
    const { service, viewService } = await mockAttached(source);

    function kinds(): Array<string> {
      return ((listMockInvokeCalls(ERenderCommand.SET_OVERLAYS).at(-1)?.overlays ?? []) as Array<{ kind: string }>).map(
        ({ kind }) => kind
      );
    }

    expect(kinds()).not.toContain(ERenderOverlay.SKELETON);

    viewService.setOptions({ ...viewService.options, isSkeletonVisible: true });
    await flush();

    expect(kinds()).toContain(ERenderOverlay.SKELETON);
    expect(kinds()).toContain(ERenderOverlay.POINTS);

    service.dispose();
  });

  // Clicking through a tree keeps the view the person turned to: only the first model a view shows is framed.
  it("frames the first model a view shows, and keeps the camera for the ones after", async () => {
    const source: IVisualRenderSource = mockSource({ model: mockVisualModelViews(), sessionId: "first" });
    const { service } = await mockAttached(source);

    function framings(): Array<number> {
      return [
        listMockInvokeCalls(ERenderCommand.SET_CAMERA).length,
        listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).filter(
          ({ command }) => (command as { kind: string }).kind === EWorldCameraCommand.RESET
        ).length,
      ];
    }

    expect(framings()).toEqual([1, 1]);

    runInAction(() => (source.model = mockVisualModelViews({ fit: { center: [0, 1, 0], radius: 4 } })));
    await flush();

    expect(framings()).toEqual([1, 1]);

    service.resetCamera();
    await flush();

    expect(framings()).toEqual([2, 2]);

    service.dispose();
  });

  it("notes what became of the model's textures once the viewport holds it whole", async () => {
    const source: IVisualRenderSource = mockSource({ model: mockVisualModelViews(), sessionId: "first" });
    const { container, service } = await mockAttached(source);
    const loadService: VisualLoadService = container.get(VisualLoadService);

    mockLoadedVisual(
      loadService,
      mockSelectedVisual({ dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] } }),
      "first"
    );
    mockIpcCommands({
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [
        {
          reference: mockTextureDependency().reference,
          state: {
            isExpanded: true,
            height: 4,
            kind: ERenderTextureState.LOADED,
            layout: "DXT5",
            levels: 1,
            width: 4,
          },
          group: ERenderTextureGroup.SURFACES,
        },
      ],
    });

    // A ready report of the model shown before, still in the channel, describes nothing.
    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: SCENE - 1 }) });
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.DESCRIBE_TEXTURE_LOADS)).toHaveLength(0);

    emit({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 0,
        isReady: true,
        scene: SCENE,
        sectors: 0,
        sectorsTotal: 0,
        textures: 1,
        texturesTotal: 1,
      }),
    });
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.DESCRIBE_TEXTURE_LOADS)).toHaveLength(1);
    expect(loadService.textureStatuses.get(0)?.state).toBe(EVisualTextureState.DECODED);

    service.dispose();
  });

  it("reads its readout off the viewport's frames", async () => {
    const { service } = await mockAttached(mockSource());

    emit({
      kind: ERenderViewportEvent.FRAME,
      report: { framesPerSecond: 60, passes: [{ gpuTime: 1, name: "g-buffer" }], staticDraws: { commands: 4 } },
    } as unknown as ViewportEvent);

    expect(service.frame.staticDraws.commands).toBe(4);
    expect(service.frame.framesPerSecond).toBe(60);
    expect(service.frame.passes).toEqual([{ gpuTime: 1, name: "g-buffer" }]);

    service.dispose();
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.DETACH_VIEWPORT)).toHaveLength(1);
  });
});
