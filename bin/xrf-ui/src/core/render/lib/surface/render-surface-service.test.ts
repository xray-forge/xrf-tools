import { describe, expect, it } from "@jest/globals";

import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";

class MockRenderSurfaceService extends RenderSurfaceService {
  public readonly mounted: Array<HTMLElement> = [];
  public unmounts: number = 0;

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

    service.detach();

    expect(service.unmounts).toBe(1);
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
});
