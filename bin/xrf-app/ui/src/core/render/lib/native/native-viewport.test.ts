import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ERenderViewportEvent } from "@xrf/ipc/types/xrf-renderer";
import { EWorldViewportEvent } from "@xrf/ipc/types/xrf-world";

import { flush } from "@/fixtures/utils/promise";

import { INativeViewportListener, NativeViewport } from "./native-viewport";

function mockListener(): jest.Mocked<INativeViewportListener> {
  return {
    onApplied: jest.fn(),
    onCamera: jest.fn(),
    onCommandFailed: jest.fn(),
    onFailed: jest.fn(),
    onFrame: jest.fn(),
    onLoad: jest.fn(),
    onWeather: jest.fn(),
  };
}

describe("NativeViewport", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockIpcCommands({ [ERenderCommand.ATTACH_VIEWPORT]: 7 });
  });

  // The app sends both crates' events as one untagged union, which the page tells apart by `kind` alone.
  it("hears no kind the renderer and the world both send", () => {
    const rendered: ReadonlySet<string> = new Set<string>(Object.values(ERenderViewportEvent));

    expect(Object.values(EWorldViewportEvent).filter((kind: string) => rendered.has(kind))).toEqual([]);
  });

  it("answers no hit for a pick that failed, telling the failure once", async () => {
    const listener: jest.Mocked<INativeViewportListener> = mockListener();
    const viewport: NativeViewport = new NativeViewport(listener);

    mockIpcCommands({
      [ERenderCommand.ATTACH_VIEWPORT]: 7,
      [ERenderCommand.PICK]: () => {
        throw new Error("no frame");
      },
    });

    await expect(viewport.pick({ isThrough: false, x: 1, y: 2 })).resolves.toBeNull();
    await expect(viewport.pick({ isThrough: false, x: 1, y: 2 })).resolves.toBeNull();

    expect(listener.onCommandFailed).toHaveBeenCalledTimes(1);
    expect(listener.onCommandFailed).toHaveBeenCalledWith("pick", expect.any(Error));
  });

  it("rejects a description that failed, so its caller can say so", async () => {
    const listener: jest.Mocked<INativeViewportListener> = mockListener();
    const viewport: NativeViewport = new NativeViewport(listener);

    mockIpcCommands({
      [ERenderCommand.ATTACH_VIEWPORT]: 7,
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: () => {
        throw new Error("thread stopped");
      },
    });

    await expect(viewport.describeTextureLoads()).rejects.toThrow("thread stopped");

    expect(listener.onCommandFailed).toHaveBeenCalledWith("describeTextureLoads", expect.any(Error));
  });

  it("answers what a released viewport stands for without asking the renderer", async () => {
    const viewport: NativeViewport = new NativeViewport(mockListener());

    await flush();
    viewport.dispose();

    await expect(viewport.measureSurfaces()).resolves.toEqual([]);
    await expect(viewport.describeProblems()).resolves.toEqual({ models: [], sectors: [], skipped: [] });
    await expect(viewport.locateSpawnObject(3)).resolves.toBeNull();
  });

  it("answers a located object's sphere as four numbers", async () => {
    const viewport: NativeViewport = new NativeViewport(mockListener());

    mockIpcCommands({ [ERenderCommand.ATTACH_VIEWPORT]: 7, [ERenderCommand.LOCATE_SPAWN_OBJECT]: [1, 2, 3, 4] });

    await expect(viewport.locateSpawnObject(3)).resolves.toEqual([1, 2, 3, 4]);
  });
});
