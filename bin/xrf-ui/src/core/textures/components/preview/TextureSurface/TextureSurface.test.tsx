import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { createRendererWorkerStub } from "@xrf/renderer/fixtures";

import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { renderWithProviders } from "@/fixtures/utils/render";

let TextureSurface: typeof import("./TextureSurface").TextureSurface;
let TextureRenderService: typeof import("@/core/textures/services/render").TextureRenderService;
let dragLight: ReturnType<typeof jest.spyOn>;

beforeAll(async () => {
  // Stubbed at the thread boundary: jsdom has neither a GPU nor an offscreen canvas.
  jest.doMock("@xrf/renderer/worker", () => ({ createRendererWorker: () => createRendererWorkerStub().worker }));
  HTMLCanvasElement.prototype.transferControlToOffscreen = function () {
    return {} as OffscreenCanvas;
  };

  ({ TextureSurface } = await import("./TextureSurface"));
  ({ TextureRenderService } = await import("@/core/textures/services/render"));
  dragLight = jest.spyOn(TextureRenderService.prototype, "dragLight");
});

function sendPointer(target: HTMLElement, type: string, options: MouseEventInit = {}): void {
  const event: MouseEvent = new MouseEvent(type, { bubbles: true, ...options });

  Object.defineProperty(event, "pointerId", { value: 1 });
  fireEvent(target, event);
}

function startDrag(): HTMLElement {
  const { getByTestId } = renderWithProviders(<TextureSurface />, {
    bindings: [TextureSelectionService, TextureSurfaceService, TextureViewService, TextureRenderService],
  });
  const surface: HTMLElement = getByTestId("texture-surface").firstElementChild as HTMLElement;
  const captured: Set<number> = new Set();

  // jsdom has no pointer capture implementation.
  Object.assign(surface, {
    setPointerCapture: (id: number) => captured.add(id),
    hasPointerCapture: (id: number) => captured.has(id),
    releasePointerCapture: (id: number) => captured.delete(id),
  });

  sendPointer(surface, "pointerdown", { shiftKey: true, clientX: 10, clientY: 10 });

  return surface;
}

describe("TextureSurface", () => {
  it("stops moving the light after pointer cancellation", () => {
    const surface: HTMLElement = startDrag();

    sendPointer(surface, "pointermove", { clientX: 20, clientY: 20 });
    expect(dragLight).toHaveBeenCalledWith(10, 10);
    dragLight.mockClear();

    sendPointer(surface, "pointercancel");
    sendPointer(surface, "pointermove", { clientX: 30, clientY: 30 });

    expect(dragLight).not.toHaveBeenCalled();
    expect(surface.hasPointerCapture(1)).toBe(false);
  });

  it("stops moving the light after the surface loses pointer capture", () => {
    const surface: HTMLElement = startDrag();

    surface.releasePointerCapture(1);
    sendPointer(surface, "lostpointercapture");
    sendPointer(surface, "pointermove", { clientX: 20, clientY: 20 });

    expect(dragLight).not.toHaveBeenCalled();
  });

  it("keeps the light drag when the child canvas loses pointer capture", () => {
    const surface: HTMLElement = startDrag();
    const canvas: HTMLCanvasElement = surface.querySelector("canvas") as HTMLCanvasElement;

    sendPointer(canvas, "lostpointercapture");
    sendPointer(surface, "pointermove", { clientX: 20, clientY: 20 });

    expect(dragLight).toHaveBeenCalledWith(10, 10);
    expect(surface.hasPointerCapture(1)).toBe(true);
  });
});
