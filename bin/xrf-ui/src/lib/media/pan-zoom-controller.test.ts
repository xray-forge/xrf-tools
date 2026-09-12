import { describe, expect, it } from "@jest/globals";

import { EPanZoomMode, IPanZoomState, PAN_ZOOM_FIT, toManualPanZoom } from "@/lib/media/pan-zoom";
import { PanZoomController } from "@/lib/media/pan-zoom-controller";

const PLACED: IPanZoomState = toManualPanZoom({ center: { x: 128, y: 64 }, scale: 2 });

describe("PanZoomController", () => {
  it("opens fitted, the way a viewport that has not been touched shows a picture", () => {
    expect(new PanZoomController().get()).toBe(PAN_ZOOM_FIT);
  });

  it("tells every viewport watching the camera it moved", () => {
    const controller: PanZoomController = new PanZoomController();
    const seen: Array<IPanZoomState> = [];

    controller.subscribe(() => seen.push(controller.get()));
    controller.subscribe(() => seen.push(controller.get()));

    controller.set(PLACED);

    // Both, and with the camera already moved: a comparison's two panes read the controller back rather than being
    // handed anything, so neither can be told before it is true.
    expect(seen).toEqual([PLACED, PLACED]);
  });

  it("derives the next camera from the current one", () => {
    const controller: PanZoomController = new PanZoomController();

    controller.set(PLACED);
    controller.set((current: IPanZoomState) =>
      current.mode === EPanZoomMode.MANUAL ? toManualPanZoom({ ...current.camera, scale: 4 }) : current
    );

    expect(controller.get()).toEqual(toManualPanZoom({ center: { x: 128, y: 64 }, scale: 4 }));
  });

  it("says nothing when the camera is set to what it already is", () => {
    const controller: PanZoomController = new PanZoomController();
    let notified: number = 0;

    controller.subscribe(() => (notified += 1));

    controller.set(PLACED);
    controller.set(PLACED);

    expect(notified).toBe(1);
  });

  it("stops telling a viewport that has gone away", () => {
    const controller: PanZoomController = new PanZoomController();
    let notified: number = 0;

    const unsubscribe: () => void = controller.subscribe(() => (notified += 1));

    controller.set(PLACED);
    unsubscribe();
    controller.set(PAN_ZOOM_FIT);

    expect(notified).toBe(1);
  });
});
