import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { TextureRenderService } from "@/core/textures/services/render";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TextureSurface } from "./TextureSurface";

let dragLight: ReturnType<typeof jest.spyOn>;

beforeAll(() => {
  dragLight = jest.spyOn(TextureRenderService.prototype, "dragLight");
});

beforeEach(() => {
  resetMockInvoke();
  setMockInvokeResponses({ ["plugin:render|attach_viewport"]: 1 });
  dragLight.mockClear();
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

  it("keeps the light drag when the viewport inside it loses pointer capture", () => {
    const surface: HTMLElement = startDrag();
    const viewport: HTMLElement = surface.firstElementChild as HTMLElement;

    sendPointer(viewport, "lostpointercapture");
    sendPointer(surface, "pointermove", { clientX: 20, clientY: 20 });

    expect(dragLight).toHaveBeenCalledWith(10, 10);
    expect(surface.hasPointerCapture(1)).toBe(true);
  });

  // The viewport pans its camera on a shift drag, which would move the body out from under the light being placed.
  it("keeps a shift drag from the viewport's camera", async () => {
    const surface: HTMLElement = startDrag();

    // A plain press over the same viewport does reach it, once the viewport has attached.
    sendPointer(surface.firstElementChild as HTMLElement, "pointerdown", { clientX: 10, clientY: 10 });

    for (let index: number = 0; index < 10; index += 1) {
      await Promise.resolve();
    }

    expect(mockInvoke.mock.calls.filter(([name]) => name === "plugin:render|send_input")).toHaveLength(1);
  });
});
