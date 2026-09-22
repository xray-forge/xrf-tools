import { describe, expect, it } from "@jest/globals";

import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";

class MockRenderSurfaceService extends RenderSurfaceService {
  public readonly mounted: Array<HTMLElement> = [];
  public unmounts: number = 0;

  public rebuild(): void {
    this.remount();
  }

  public get hasSurface(): boolean {
    return this.isAttached;
  }

  protected mount(container: HTMLElement): void {
    this.mounted.push(container);
  }

  protected unmount(): void {
    this.unmounts += 1;
  }
}

describe("RenderSurfaceService", () => {
  it("builds against the element it was given, and releases it once", () => {
    const service: MockRenderSurfaceService = new MockRenderSurfaceService();
    const container: HTMLElement = document.createElement("div");

    service.attach(container);

    expect(service.mounted).toEqual([container]);
    expect(service.hasSurface).toBe(true);

    service.detach();

    expect(service.unmounts).toBe(1);
    expect(service.hasSurface).toBe(false);
  });

  // Strict mode mounts an effect twice, and a view may hand over a different element at any time.
  it("releases what it had before building somewhere else", () => {
    const service: MockRenderSurfaceService = new MockRenderSurfaceService();
    const first: HTMLElement = document.createElement("div");
    const second: HTMLElement = document.createElement("div");

    service.attach(first);
    service.attach(second);

    expect(service.mounted).toEqual([first, second]);
    expect(service.unmounts).toBe(1);
  });

  it("releases nothing twice", () => {
    const service: MockRenderSurfaceService = new MockRenderSurfaceService();

    service.attach(document.createElement("div"));
    service.detach();
    service.detach();

    expect(service.unmounts).toBe(1);
  });

  // What a rebuild is for: the answer to "how should this draw" changed, and only building again can apply it.
  it("builds again against the same element", () => {
    const service: MockRenderSurfaceService = new MockRenderSurfaceService();
    const container: HTMLElement = document.createElement("div");

    service.attach(container);
    service.rebuild();

    expect(service.mounted).toEqual([container, container]);
    expect(service.unmounts).toBe(1);
  });

  // The next attach builds whatever the new answer is, so there is nothing to do here and nothing to remember.
  it("does not build while nothing is attached", () => {
    const service: MockRenderSurfaceService = new MockRenderSurfaceService();

    service.rebuild();

    expect(service.mounted).toEqual([]);
    expect(service.unmounts).toBe(0);
  });
});
