import { describe, expect, it } from "@jest/globals";

import {
  clampScale,
  IPanZoomCamera,
  IPanZoomPoint,
  IPanZoomSize,
  IPanZoomState,
  IPanZoomTransform,
  PAN_ZOOM_FIT,
  PAN_ZOOM_MAXIMUM_SCALE,
  PAN_ZOOM_MINIMUM_SCALE,
  panBy,
  resolvePanZoomCamera,
  toContentPoint,
  toFitCamera,
  toManualPanZoom,
  toPanZoomTransform,
  zoomAround,
  zoomByWheel,
} from "@/lib/media/pan-zoom";

const CONTENT: IPanZoomSize = { width: 1024, height: 512 };
const VIEWPORT: IPanZoomSize = { width: 800, height: 600 };

/**
 * Projects a content-space coordinate into the viewport, the way the rendered transform does.
 *
 * @param state - What the viewport is showing.
 * @param point - Content-space coordinate to project.
 * @param viewport - Viewport size to project it into.
 * @returns The projected viewport coordinate.
 */
function project(state: IPanZoomState, point: IPanZoomPoint, viewport: IPanZoomSize = VIEWPORT): IPanZoomPoint {
  const transform: IPanZoomTransform = toPanZoomTransform(state, CONTENT, viewport);

  return { x: transform.offsetX + point.x * transform.scale, y: transform.offsetY + point.y * transform.scale };
}

describe("pan-zoom", () => {
  it("keeps the anchored point exactly where it was", () => {
    const camera: IPanZoomCamera = { center: { x: 400, y: 200 }, scale: 1 };
    const anchor: IPanZoomPoint = toContentPoint(camera, VIEWPORT, { x: 120, y: 80 });

    // Whatever content sits under the cursor before the zoom has to sit under it afterwards; this is the whole reason
    // for anchoring rather than scaling about the centre.
    const before: IPanZoomPoint = project(toManualPanZoom(camera), anchor);
    const after: IPanZoomPoint = project(toManualPanZoom(zoomAround(camera, anchor, 4)), anchor);

    expect(after.x).toBeCloseTo(before.x);
    expect(after.y).toBeCloseTo(before.y);
    expect(zoomAround(camera, anchor, 4).scale).toBe(4);
  });

  it("holds the anchor across repeated wheel notches", () => {
    let camera: IPanZoomCamera = { center: { x: 512, y: 256 }, scale: 1 };

    const anchor: IPanZoomPoint = toContentPoint(camera, VIEWPORT, { x: 200, y: 140 });
    const before: IPanZoomPoint = project(toManualPanZoom(camera), anchor);

    for (let index = 0; index < 8; index += 1) {
      camera = zoomByWheel(camera, anchor, -1);
    }

    const after: IPanZoomPoint = project(toManualPanZoom(camera), anchor);

    // Drift here would be invisible on one notch and obvious after a few, which is how it would ship.
    expect(after.x).toBeCloseTo(before.x);
    expect(after.y).toBeCloseTo(before.y);
    expect(camera.scale).toBeGreaterThan(1);
  });

  it("zooms out on a downward wheel and in on an upward one", () => {
    const camera: IPanZoomCamera = { center: { x: 0, y: 0 }, scale: 1 };

    expect(zoomByWheel(camera, camera.center, 1).scale).toBeLessThan(1);
    expect(zoomByWheel(camera, camera.center, -1).scale).toBeGreaterThan(1);
  });

  it("clamps the scale at both ends", () => {
    const camera: IPanZoomCamera = { center: { x: 0, y: 0 }, scale: 1 };

    expect(clampScale(1000)).toBe(PAN_ZOOM_MAXIMUM_SCALE);
    expect(clampScale(0)).toBe(PAN_ZOOM_MINIMUM_SCALE);
    expect(zoomAround(camera, camera.center, 1000).scale).toBe(PAN_ZOOM_MAXIMUM_SCALE);
  });

  it("pans the content by exactly the delta it is given", () => {
    const camera: IPanZoomCamera = { center: { x: 512, y: 256 }, scale: 3 };
    const panned: IPanZoomState = toManualPanZoom(panBy(camera, -5, 7));

    const before: IPanZoomPoint = project(toManualPanZoom(camera), camera.center);
    const after: IPanZoomPoint = project(panned, camera.center);

    // The camera moves against the drag, so the content moves with it, by the drag and no more.
    expect(after.x - before.x).toBeCloseTo(-5);
    expect(after.y - before.y).toBeCloseTo(7);
  });

  it("fits a large image and centres it", () => {
    const transform: IPanZoomTransform = toPanZoomTransform(PAN_ZOOM_FIT, CONTENT, { width: 512, height: 512 });

    expect(transform.scale).toBe(0.5);
    expect(transform.offsetX).toBe(0);
    // Letterboxed vertically, so the remaining space is split evenly.
    expect(transform.offsetY).toBe((512 - 512 * 0.5) / 2);
  });

  it("never enlarges something smaller than the viewport", () => {
    // Blowing a 16px icon up to fill the pane says less about it than seeing how small it is.
    expect(toFitCamera({ width: 16, height: 16 }, { width: 512, height: 512 }).scale).toBe(1);
  });

  it("places nothing while a dimension is unknown", () => {
    const unplaced: IPanZoomTransform = { scale: 1, offsetX: 0, offsetY: 0 };

    expect(toPanZoomTransform(PAN_ZOOM_FIT, { width: 0, height: 0 }, VIEWPORT)).toEqual(unplaced);
    expect(toPanZoomTransform(PAN_ZOOM_FIT, CONTENT, { width: 0, height: 0 })).toEqual(unplaced);
  });

  it("holds a manual camera still through a resize", () => {
    const camera: IPanZoomCamera = { center: { x: 300, y: 140 }, scale: 2.5 };
    const state: IPanZoomState = toManualPanZoom(camera);

    const small: IPanZoomPoint = project(state, camera.center, { width: 640, height: 480 });
    const large: IPanZoomPoint = project(state, camera.center, { width: 1600, height: 900 });

    // What the person centred stays centred, whatever the window did around it. Holding a pixel offset instead is what
    // used to walk the picture out of the pane on every fullscreen toggle.
    expect(small).toEqual({ x: 320, y: 240 });
    expect(large).toEqual({ x: 800, y: 450 });

    const corner: IPanZoomPoint = { x: 0, y: 0 };
    const fromSmall: IPanZoomPoint = project(state, corner, { width: 640, height: 480 });
    const fromLarge: IPanZoomPoint = project(state, corner, { width: 1600, height: 900 });

    // And everything else keeps its distance from the centre, so the resize moves the frame and not the picture.
    expect(fromLarge.x - large.x).toBeCloseTo(fromSmall.x - small.x);
    expect(fromLarge.y - large.y).toBeCloseTo(fromSmall.y - small.y);
  });

  it("re-fits an untouched picture into the size the viewport becomes", () => {
    const small: IPanZoomTransform = toPanZoomTransform(PAN_ZOOM_FIT, CONTENT, { width: 512, height: 512 });
    const large: IPanZoomTransform = toPanZoomTransform(PAN_ZOOM_FIT, CONTENT, { width: 2048, height: 2048 });

    // A fit is a rule and not a snapshot, so growing the pane shows more of the picture rather than the same crop
    // pinned to a corner.
    expect(small.scale).toBe(0.5);
    expect(large.scale).toBe(1);
    expect(project(PAN_ZOOM_FIT, { x: 512, y: 256 }, { width: 2048, height: 2048 })).toEqual({ x: 1024, y: 1024 });
  });

  it("resolves a fit into the camera a gesture then moves", () => {
    const fitted: IPanZoomCamera = resolvePanZoomCamera(PAN_ZOOM_FIT, CONTENT, VIEWPORT);

    // The first drag on an untouched picture has to start from where that picture actually is, not from the origin.
    expect(fitted).toEqual(toFitCamera(CONTENT, VIEWPORT));
    expect(resolvePanZoomCamera(toManualPanZoom(fitted), CONTENT, { width: 10, height: 10 })).toBe(fitted);
  });
});
