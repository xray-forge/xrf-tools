import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { EMPTY_RENDER_FRAME_COST, IRenderFrameCost } from "@xrf/renderer";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { SettingsService } from "@/core/settings/services/settings";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/scene/texture-lighting";
import { DEFAULT_TEXTURE_PREVIEW_OPTIONS, ETexturePreviewMode } from "@/core/textures/lib/texture-preview";
import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { mockContainer } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

const scene = {
  dispose: jest.fn(),
  setReporter: jest.fn(),
  dolly: jest.fn(),
  dragLight: jest.fn<(deltaX: number, deltaY: number) => IRenderLighting>(),
  reset: jest.fn(),
  setFrameRateLimit: jest.fn(),
  setLighting: jest.fn(),
  setOptions: jest.fn(),
  setTextures: jest.fn(),
};

let TextureRenderService: typeof import("./texture-render.service").TextureRenderService;

beforeAll(async () => {
  // Stubbed at the GPU boundary; jsdom cannot construct a WebGL renderer, and nothing below it is under test.
  jest.doMock("@/core/textures/lib/scene/TextureSurfaceScene", () => ({
    TextureSurfaceScene: jest.fn(() => scene),
  }));

  ({ TextureRenderService } = await import("./texture-render.service"));
});

function mockAttached(): {
  container: Container;
  service: InstanceType<typeof TextureRenderService>;
  viewService: TextureViewService;
} {
  const container: Container = mockContainer([TextureViewService, TextureSurfaceService, TextureRenderService]);

  return {
    container,
    service: container.get(TextureRenderService),
    viewService: container.get(TextureViewService),
  };
}

describe("TextureRenderService", () => {
  // A scene attached after a texture was chosen would otherwise show an empty body until something happened to
  // change, which for a texture nobody is touching is never.
  it("tells a new scene what is already open", () => {
    const { service, viewService } = mockAttached();

    viewService.setOptions({ ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, mode: ETexturePreviewMode.SURFACE });
    service.attach(document.createElement("div"));

    expect(scene.setOptions).toHaveBeenCalledWith(viewService.options);
    expect(scene.setLighting).toHaveBeenCalledWith(DEFAULT_TEXTURE_LIGHTING);
    expect(scene.setTextures).toHaveBeenCalledTimes(1);
    expect(scene.setFrameRateLimit).toHaveBeenCalledTimes(1);

    service.detach();
  });

  it("carries a change to the open texture into the scene", () => {
    const { service, container } = mockAttached();
    const uploaded: ITextureSurfaceFiles = { ...EMPTY_TEXTURE_SURFACE, aspect: 2 };

    service.attach(document.createElement("div"));
    scene.setTextures.mockClear();

    runInAction(() => {
      container.get(TextureSurfaceService).files = AsyncState.ready(uploaded);
    });

    expect(scene.setTextures).toHaveBeenCalledWith(uploaded);

    service.detach();
  });

  it("releases the scene and stops carrying anything to it", () => {
    const { service, viewService } = mockAttached();

    service.attach(document.createElement("div"));
    service.detach();

    expect(scene.dispose).toHaveBeenCalled();

    scene.setLighting.mockClear();
    viewService.setLighting({ ...DEFAULT_TEXTURE_LIGHTING, sunIntensity: 9 });

    expect(scene.setLighting).not.toHaveBeenCalled();
  });

  // The drag is gathered over the element and applied by the scene, but what it swung to is the view's to keep:
  // the toolbar shows the same number the body is lit by.
  it("keeps what a drag over the body swung the light to", () => {
    const { service, viewService } = mockAttached();
    const swung: IRenderLighting = { ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: 123 };

    service.attach(document.createElement("div"));
    scene.dragLight.mockReturnValueOnce(swung);
    service.dragLight(10, 4);

    expect(scene.dragLight).toHaveBeenCalledWith(10, 4);
    expect(viewService.lighting).toBe(swung);

    service.detach();
  });

  // The readout over the viewport is the only place the answer to "which thread drew this" can be seen.
  it("says what its frames cost and where they were drawn", () => {
    const { service } = mockAttached();

    scene.setReporter.mockClear();
    service.attach(document.createElement("div"));

    expect(service.isOffscreen).toBe(false);
    expect(service.frameCost).toBe(EMPTY_RENDER_FRAME_COST);

    const report = scene.setReporter.mock.calls[0][0] as (cost: IRenderFrameCost) => void;

    report({ ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 });

    expect(service.frameCost.framesPerSecond).toBe(144);

    service.detach();

    // Nothing is drawing, so the readout says nothing rather than the last thing it saw.
    expect(service.frameCost).toBe(EMPTY_RENDER_FRAME_COST);
  });

  // Which thread a viewport draws on is decided when it is built, so the setting is answered by building again.
  it("builds again when the thread setting changes", () => {
    const { service, container } = mockAttached();

    service.attach(document.createElement("div"));

    const built: number = scene.dispose.mock.calls.length;

    container.get(SettingsService).setOffscreenRenderEnabled(false);

    expect(scene.dispose.mock.calls).toHaveLength(built + 1);

    service.detach();
  });

  // Whoever put the canvas on the page is the one that can take it off: the viewport draws on a target it was
  // handed, and one path used to leave the element behind, so a remount stacked a second canvas on the first.
  it("leaves no canvas behind when it is detached", () => {
    const { service } = mockAttached();
    const container: HTMLElement = document.createElement("div");

    service.attach(container);

    expect(container.querySelectorAll("canvas")).toHaveLength(1);

    service.attach(container);

    expect(container.querySelectorAll("canvas")).toHaveLength(1);

    service.detach();

    expect(container.querySelectorAll("canvas")).toHaveLength(0);
  });

  it("answers the viewport controls before anything is attached", () => {
    const { service } = mockAttached();

    expect(() => {
      service.dolly(2);
      service.reset();
      service.dragLight(1, 1);
      service.detach();
    }).not.toThrow();
  });
});
