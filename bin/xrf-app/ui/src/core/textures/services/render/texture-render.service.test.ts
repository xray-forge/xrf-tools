import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { ETexturesCommand } from "@xrf/ipc/commands/textures";
import {
  getMockChannels,
  listMockInvokeCalls,
  MockChannel,
  mockIpcCommands,
  resetMockChannels,
  resetMockInvoke,
} from "@xrf/ipc/testing";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";
import { ETextureSurfaceAlpha, ETextureSurfaceShape } from "@xrf/ipc/types/xrf-app-textures";
import {
  ERenderTextureGroup,
  ERenderTextureState,
  ERenderViewportEvent,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";
import { EWorldCameraCommand } from "@xrf/ipc/types/xrf-world";

import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/core/textures/fixtures/texture.mocks";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/texture-lighting";
import { TextureRenderService } from "@/core/textures/services/render";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureViewService } from "@/core/textures/services/view";
import { mockContainer } from "@/fixtures/utils/container";

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function emit(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}

/** The scene the mocked renderer draws every texture shown as. */
const SCENE: number = 6;

async function mockAttached(): Promise<{ container: Container; service: TextureRenderService }> {
  mockIpcCommands({ [ERenderCommand.ATTACH_VIEWPORT]: 5, [ETexturesCommand.SHOW_TEXTURE]: SCENE });

  const container: Container = mockContainer([TextureSelectionService, TextureViewService, TextureRenderService]);

  runInAction(() => {
    container.get(TextureSelectionService).selected = AsyncState.ready(mockTextureDescription());
  });

  const service: TextureRenderService = container.get(TextureRenderService);

  service.attach(document.createElement("div"));
  await flush();

  return { container, service };
}

describe("TextureRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
  });

  it("starts nothing until a view attaches", () => {
    mockContainer([TextureSelectionService, TextureViewService, TextureRenderService]).get(TextureRenderService);

    expect(listMockInvokeCalls(ERenderCommand.ATTACH_VIEWPORT)).toHaveLength(0);
  });

  it("shows the selected texture on the body its view asks for, and again as either changes", async () => {
    const { container, service } = await mockAttached();
    const viewService: TextureViewService = container.get(TextureViewService);

    viewService.setOptions({
      ...viewService.options,
      alpha: ETextureSurfaceAlpha.CUT_OUT,
      shape: ETextureSurfaceShape.SPHERE,
    });
    await flush();

    const requests = listMockInvokeCalls(ETexturesCommand.SHOW_TEXTURE).map(
      ({ request }) => request as Record<string, unknown>
    );

    expect(requests).toHaveLength(2);
    expect(requests[0]).toMatchObject({ shape: ETextureSurfaceShape.PLANE, source: { reference: MOCK_TEXTURE } });
    expect(requests[1]).toMatchObject({ alpha: ETextureSurfaceAlpha.CUT_OUT, shape: ETextureSurfaceShape.SPHERE });

    runInAction(() => {
      container.get(TextureSelectionService).selected = AsyncState.idle();
    });
    await flush();

    expect(listMockInvokeCalls(ETexturesCommand.SHOW_TEXTURE).at(-1)?.request).toBeNull();

    service.dispose();
  });

  it("draws the body against the alpha checkerboard, under the view's light", async () => {
    const { service } = await mockAttached();
    const options = listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options as RenderViewOptions;

    expect(options.asset.backdropSquares).not.toBeNull();
    expect(options.asset.lighting).toMatchObject({ sunAzimuth: DEFAULT_TEXTURE_LIGHTING.sunAzimuth });
    expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(1);

    service.dispose();
  });

  // What the drag swung to is the view's to keep: the toolbar shows the number the body is lit by.
  it("keeps what a drag over the body swung the light to", async () => {
    const { container, service } = await mockAttached();

    service.dragLight(1, 0);

    expect(container.get(TextureViewService).lighting.sunAzimuth).not.toBe(DEFAULT_TEXTURE_LIGHTING.sunAzimuth);

    service.dispose();
  });

  it("says why the texture cannot be laid on the body, once the renderer has read it", async () => {
    const { service } = await mockAttached();

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [
        {
          reference: MOCK_TEXTURE,
          state: { kind: ERenderTextureState.FAILED, reason: "Unknown pixel format" },
          group: ERenderTextureGroup.SURFACES,
        },
      ],
    });
    // A ready report of the texture shown before, still in the channel, asks nothing.
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

    expect(service.baseFailure).toBe("Unknown pixel format");

    service.dispose();
  });

  it("says what its frames cost, and nothing once no view is drawing", async () => {
    const { service } = await mockAttached();

    emit({
      kind: ERenderViewportEvent.FRAME,
      report: { framesPerSecond: 144, passes: [], staticDraws: { commands: 2 } },
    } as unknown as ViewportEvent);

    expect(service.frame.framesPerSecond).toBe(144);

    service.detach();

    expect(service.frame.framesPerSecond).toBe(0);

    service.dispose();
  });

  it("answers the viewport controls before anything is attached, and sends them once it is", async () => {
    const container: Container = mockContainer([TextureSelectionService, TextureViewService, TextureRenderService]);
    const unattached: TextureRenderService = container.get(TextureRenderService);

    expect(() => unattached.dolly(2)).not.toThrow();
    expect(() => unattached.reset()).not.toThrow();

    const { service } = await mockAttached();

    service.reset();
    await flush();

    expect(
      listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).map(({ command }) => (command as { kind: string }).kind)
    ).toEqual([EWorldCameraCommand.RESET]);

    service.dispose();
  });
});
