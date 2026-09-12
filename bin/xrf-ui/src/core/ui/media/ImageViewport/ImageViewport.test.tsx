import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult } from "@testing-library/react";

import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { renderWithProviders } from "@/fixtures/utils/render";
import { IPanZoomTransform } from "@/lib/media/pan-zoom";
import { PanZoomController } from "@/lib/media/pan-zoom-controller";
import { Nullable } from "@/lib/types/general";

/** The picture under test, wider than it is tall so a fit letterboxes it and the centring is visible. */
const WIDTH: number = 1024;
const HEIGHT: number = 512;

/** What 1024x512 fits to in an 800x600 pane: the width binds, and the 400px of height left over is split evenly. */
const FITTED: IPanZoomTransform = { scale: 0.78125, offsetX: 0, offsetY: 100 };

/** What every mounted element reports, since jsdom lays nothing out and answers zero for both dimensions. */
let laidOut = { width: 800, height: 600 };

/** The observer's callback, so a test can resize the viewport the way a window does. */
let notify: Nullable<() => void> = null;

class TestResizeObserver {
  public constructor(callback: () => void) {
    notify = callback;
  }

  public observe(): void {}

  public unobserve(): void {}

  public disconnect(): void {
    notify = null;
  }
}

/**
 * Mounts a viewport over the picture under test.
 *
 * @returns The render, and the box that takes the gestures.
 */
function renderViewport(): { viewport: HTMLElement; render: RenderResult } {
  const render: RenderResult = renderWithProviders(
    <ImageViewport src={"texture.png"} alt={"texture"} width={WIDTH} height={HEIGHT} />
  );

  return { viewport: render.getByAltText("texture").parentElement as HTMLElement, render };
}

/**
 * Reads where the picture currently sits.
 *
 * @param render - Render holding the picture.
 * @returns The transform it is drawn with.
 */
function readTransform(render: RenderResult): IPanZoomTransform {
  const written: string = getComputedStyle(render.getByAltText("texture")).transform;
  const numbers: Array<number> = (written.match(/-?[\d.]+/g) ?? []).map(Number);

  return { offsetX: numbers[0], offsetY: numbers[1], scale: numbers[2] };
}

/**
 * Resizes every mounted element, the way a maximise or a fullscreen toggle does.
 *
 * @param width - New width in css pixels.
 * @param height - New height in css pixels.
 */
function resize(width: number, height: number): void {
  act(() => {
    laidOut = { width, height };
    notify?.();
  });
}

/**
 * Drags across the viewport, in one move.
 *
 * @param viewport - Box taking the gesture.
 * @param deltaX - Horizontal movement in viewport pixels.
 * @param deltaY - Vertical movement in viewport pixels.
 */
function drag(viewport: HTMLElement, deltaX: number, deltaY: number): void {
  fireEvent.mouseDown(viewport, { clientX: 100, clientY: 100 });
  fireEvent.mouseMove(viewport, { clientX: 100 + deltaX, clientY: 100 + deltaY });
  fireEvent.mouseUp(viewport);
}

describe("ImageViewport", () => {
  beforeEach(() => {
    laidOut = { width: 800, height: 600 };
    notify = null;

    Object.defineProperty(HTMLElement.prototype, "clientWidth", { configurable: true, get: () => laidOut.width });
    Object.defineProperty(HTMLElement.prototype, "clientHeight", { configurable: true, get: () => laidOut.height });

    global.ResizeObserver = TestResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    Reflect.deleteProperty(HTMLElement.prototype, "clientWidth");
    Reflect.deleteProperty(HTMLElement.prototype, "clientHeight");
  });

  it("opens the picture fitted and centred", () => {
    const { render } = renderViewport();

    expect(readTransform(render)).toEqual(FITTED);
  });

  it("re-fits an untouched picture into the size the window becomes", () => {
    const { render } = renderViewport();

    resize(1600, 900);

    // A fit is a rule and not a snapshot, so a maximised window shows the picture larger and still centred, rather
    // than the same crop left behind in a corner.
    expect(readTransform(render)).toEqual({ scale: 1, offsetX: 800 - WIDTH / 2, offsetY: 450 - HEIGHT / 2 });
  });

  it("leaves a picture someone placed exactly where they left it", () => {
    const { render, viewport } = renderViewport();

    drag(viewport, 50, -30);

    const placed: IPanZoomTransform = readTransform(render);

    expect(placed.offsetX).toBeCloseTo(FITTED.offsetX + 50);
    expect(placed.offsetY).toBeCloseTo(FITTED.offsetY - 30);

    resize(1600, 900);

    const resized: IPanZoomTransform = readTransform(render);

    // The whole point: the centre of the pane moved by (400, 150) and the picture went with it, at the magnification
    // it was left at. Holding the css offset instead is what walked the picture out of the pane on every toggle.
    expect(resized.scale).toBe(placed.scale);
    expect(resized.offsetX).toBeCloseTo(placed.offsetX + 400);
    expect(resized.offsetY).toBeCloseTo(placed.offsetY + 150);
  });

  it("puts a placed picture back to the fit on request", () => {
    const { render, viewport } = renderViewport();

    drag(viewport, 300, 300);
    fireEvent.click(render.getByLabelText("Fit to view"));

    expect(readTransform(render)).toEqual(FITTED);
  });

  it("places the picture without restyling it", () => {
    const { render, viewport } = renderViewport();

    const styled: string = render.getByAltText("texture").className;

    drag(viewport, 40, 25);

    // The class is what Emotion mints a css rule for. A transform carried through `sx` mints one per frame of every
    // drag, and the document keeps them, so the style recalculation behind a pan grows for as long as the window is
    // open. The transform is written to the element instead, and the class it is written on never changes.
    expect(render.getByAltText("texture").className).toBe(styled);
    expect(readTransform(render).offsetX).toBeCloseTo(FITTED.offsetX + 40);
  });

  it("moves two viewports sharing one camera together", () => {
    const controller: PanZoomController = new PanZoomController();

    const render: RenderResult = renderWithProviders(
      <>
        <ImageViewport src={"on-disk.png"} alt={"on disk"} width={WIDTH} height={HEIGHT} controller={controller} />
        <ImageViewport
          src={"would-write.png"}
          alt={"would write"}
          width={WIDTH}
          height={HEIGHT}
          controller={controller}
          hasControls={false}
        />
      </>
    );

    const first: HTMLElement = render.getByAltText("on disk").parentElement as HTMLElement;

    fireEvent.mouseDown(first, { clientX: 100, clientY: 100 });
    fireEvent.mouseMove(first, { clientX: 160, clientY: 80 });
    fireEvent.mouseUp(first);

    // Panning one picture of a comparison to a corner and finding the other still centred is the one thing a pair must
    // not do, and neither viewport re-renders to keep up: each writes its own element when the camera moves.
    expect(getComputedStyle(render.getByAltText("would write")).transform).toBe(
      getComputedStyle(render.getByAltText("on disk")).transform
    );
  });

  it("opens the next picture fitted rather than under the last one's camera", () => {
    const { render, viewport } = renderViewport();

    drag(viewport, 600, 400);

    render.rerender(<ImageViewport src={"other.png"} alt={"texture"} width={WIDTH} height={HEIGHT} />);

    // Inheriting the pan would open the next texture somewhere off screen, which reads as a preview that failed.
    expect(readTransform(render)).toEqual(FITTED);
  });
});
