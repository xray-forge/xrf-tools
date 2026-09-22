import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { renderWithProviders } from "@/fixtures/utils/render";

const dragLight = jest.fn<(deltaX: number, deltaY: number) => void>();

let TextureSurface: typeof import("./TextureSurface").TextureSurface;
// Imported with the component rather than above it: the service is what builds the scene, so it has to be the
// stubbed one too.
let TextureRenderService: typeof import("@/core/textures/services/render").TextureRenderService;

beforeAll(async () => {
  // Load the component after stubbing its GPU boundary; jsdom cannot construct a WebGL renderer.
  jest.doMock("@/core/textures/lib/scene/TextureSurfaceScene", () => ({
    TextureSurfaceScene: jest.fn(() => ({
      dispose: jest.fn(),
      setTextures: jest.fn(),
      setOptions: jest.fn(),
      setFrameRateLimit: jest.fn(),
      setLighting: jest.fn(),
      dragLight,
    })),
  }));

  ({ TextureSurface } = await import("./TextureSurface"));
  ({ TextureRenderService } = await import("@/core/textures/services/render"));
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
