import { describe, expect, it, jest } from "@jest/globals";
import { OffscreenRenderTarget } from "@xrf/renderer";

import { ERenderFrame, ERenderFrameResponse } from "@/core/render/lib/worker/render-frame-messages";
import { ERenderInput, IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import { IRenderWorkerScene, RenderWorkerHost } from "@/core/render/lib/worker/render-worker-host";

interface IMockRequest {
  kind: "draw";
}

function mockScene(): IRenderWorkerScene<IMockRequest> & { taken: Array<IMockRequest> } {
  return {
    dispose: jest.fn(),
    taken: [],
    take(request: IMockRequest) {
      this.taken.push(request);
    },
  };
}

function mockHost(): {
  host: RenderWorkerHost<IMockRequest>;
  said: Array<{ kind: string }>;
  built: Array<{ scene: ReturnType<typeof mockScene>; target: OffscreenRenderTarget; element: RenderProxyElement }>;
} {
  const said: Array<{ kind: string }> = [];
  const built: Array<{
    scene: ReturnType<typeof mockScene>;
    target: OffscreenRenderTarget;
    element: RenderProxyElement;
  }> = [];

  const host: RenderWorkerHost<IMockRequest> = new RenderWorkerHost(
    (target: OffscreenRenderTarget, element: RenderProxyElement) => {
      const scene = mockScene();

      built.push({ element, scene, target });

      return scene;
    },
    (response) => said.push(response)
  );

  return { built, host, said };
}

function start(host: RenderWorkerHost<IMockRequest>, height: number = 540): void {
  host.take({ canvas: {} as OffscreenCanvas, height, kind: ERenderFrame.START, pixelRatio: 1, width: 960 });
}

function mockInput(): IRenderInputEvent {
  return {
    altKey: false,
    button: 0,
    buttons: 1,
    clientX: 0,
    clientY: 0,
    ctrlKey: false,
    deltaMode: 0,
    deltaX: 0,
    deltaY: 0,
    isPrimary: true,
    metaKey: false,
    pointerId: 1,
    pointerType: "mouse",
    shiftKey: false,
    type: ERenderInput.POINTER_DOWN,
  };
}

describe("RenderWorkerHost", () => {
  // Every message may be posted before the start is handled, and a host that threw on one would take the
  // whole thread down rather than draw one frame late.
  it("drops anything said before there is a canvas", () => {
    const { host, said, built } = mockHost();

    expect(() => {
      host.take({ kind: "draw" });
      host.take({ height: 1, kind: ERenderFrame.RESIZE, pixelRatio: 1, width: 1 });
      host.take({ event: mockInput(), kind: ERenderFrame.INPUT });
      host.take({ kind: ERenderFrame.DISPOSE });
    }).not.toThrow();

    expect(said).toEqual([]);
    expect(built).toEqual([]);
  });

  it("builds the viewport on the canvas it was handed, and hands it its own messages", () => {
    const { host, built } = mockHost();

    start(host);
    host.take({ kind: "draw" });

    expect(built).toHaveLength(1);
    expect(built[0].target.height).toBe(540);
    expect(built[0].scene.taken).toEqual([{ kind: "draw" }]);
  });

  // The element is the size of the canvas, and controls read it to scale what a drag is worth.
  it("resizes the canvas and the stand-in for it together", () => {
    const { host, built } = mockHost();

    start(host);
    host.take({ height: 200, kind: ERenderFrame.RESIZE, pixelRatio: 2, width: 400 });

    expect(built[0].target.height).toBe(200);
    expect(built[0].target.pixelRatio).toBe(2);
    expect(built[0].element.clientHeight).toBe(200);
  });

  it("hands a gesture to the stand-in rather than to the viewport", () => {
    const { host, built } = mockHost();
    const listener = jest.fn();

    start(host);
    built[0].element.addEventListener(ERenderInput.POINTER_DOWN, listener);
    host.take({ event: mockInput(), kind: ERenderFrame.INPUT });

    expect(listener).toHaveBeenCalledTimes(1);
    expect(built[0].scene.taken).toEqual([]);
  });

  // Three writes the cursor on the element it was given, and only the page has one to show it on.
  it("hands the cursor back to the side that has one", () => {
    const { host, built, said } = mockHost();

    start(host);
    built[0].element.style.cursor = "grabbing";

    expect(said).toEqual([{ cursor: "grabbing", kind: ERenderFrameResponse.CURSOR }]);
  });

  it("releases the viewport, and says nothing more to it", () => {
    const { host, built } = mockHost();

    start(host);
    host.take({ kind: ERenderFrame.DISPOSE });

    expect(built[0].scene.dispose).toHaveBeenCalledTimes(1);

    host.take({ kind: "draw" });

    expect(built[0].scene.taken).toEqual([]);
  });

  // A second canvas is what a rebuild looks like from here, and the first viewport is not left running.
  it("releases the one it had before building another", () => {
    const { host, built } = mockHost();

    start(host);
    start(host, 300);

    expect(built).toHaveLength(2);
    expect(built[0].scene.dispose).toHaveBeenCalledTimes(1);
    expect(built[1].scene.dispose).not.toHaveBeenCalled();
  });
});
