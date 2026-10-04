import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ETextureSurfaceAlpha, ETextureSurfaceShape } from "@/core/ipc/types/xrf-app";
import {
  ERenderCameraCommand,
  ERenderTextureState,
  ERenderViewportEvent,
  RenderViewportEvent,
} from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/texture-lighting";
import { TextureRenderService } from "@/core/textures/services/render";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureViewService } from "@/core/textures/services/view";
import { mockRenderLoadReport } from "@/fixtures/mocks/render.mocks";
import {
  getMockChannels,
  MockChannel,
  mockInvoke,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

/** What a render command was sent, in order. */
function sent(command: string): Array<Record<string, unknown>> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === `plugin:render|${command}`)
    .map(([, args]) => args as Record<string, unknown>);
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function emit(event: RenderViewportEvent): void {
  (getMockChannels()[0] as MockChannel<RenderViewportEvent>).onmessage(event);
}

async function mockAttached(): Promise<{ container: Container; service: TextureRenderService }> {
  setMockInvokeResponses({ ["plugin:render|attach_viewport"]: 5 });

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

    expect(sent("attach_viewport")).toHaveLength(0);
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

    const requests = sent("show_texture").map(({ request }) => request as Record<string, unknown>);

    expect(requests).toHaveLength(2);
    expect(requests[0]).toMatchObject({ shape: ETextureSurfaceShape.PLANE, source: { reference: MOCK_TEXTURE } });
    expect(requests[1]).toMatchObject({ alpha: ETextureSurfaceAlpha.CUT_OUT, shape: ETextureSurfaceShape.SPHERE });

    runInAction(() => {
      container.get(TextureSelectionService).selected = AsyncState.idle();
    });
    await flush();

    expect(sent("show_texture").at(-1)?.request).toBeNull();

    service.dispose();
  });

  it("draws the body against the alpha checkerboard, under the view's light", async () => {
    const { service } = await mockAttached();
    const options = sent("set_view_options").at(-1)?.options as Record<string, unknown>;

    expect(options.backdropSquares).not.toBeNull();
    expect(options.assetLighting).toMatchObject({ sunAzimuth: DEFAULT_TEXTURE_LIGHTING.sunAzimuth });
    expect(sent("set_camera")).toHaveLength(1);

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

    setMockInvokeResponses({
      ["plugin:render|describe_textures"]: [
        { reference: MOCK_TEXTURE, state: { kind: ERenderTextureState.FAILED, reason: "Unknown pixel format" } },
      ],
    });
    emit({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 0,
        isReady: true,
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
    } as unknown as RenderViewportEvent);

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

    expect(sent("command_camera").map(({ command }) => (command as { kind: string }).kind)).toEqual([
      ERenderCameraCommand.RESET,
    ]);

    service.dispose();
  });
});
