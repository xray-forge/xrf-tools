/** A point, in whichever space the function taking it names. */
export interface IPanZoomPoint {
  x: number;
  y: number;
}

/** An extent in pixels - the content's own, or the viewport it is shown in. */
export interface IPanZoomSize {
  width: number;
  height: number;
}

/**
 * Where a viewport is looking, and how magnified.
 *
 * A camera rather than a corner offset on purpose. An offset is a distance from the top left of a box, so it only means
 * what it meant while that box keeps its size: resize the box and the same numbers put the content somewhere else,
 * which is how a maximised window loses the picture. A content-space centre and a scale mean the same thing at every
 * size, so resizing is nothing to handle - the offset is derived again from whatever the box is now.
 */
export interface IPanZoomCamera {
  /** Content-space point held at the centre of the viewport. */
  center: IPanZoomPoint;
  /** Viewport pixels per content pixel. */
  scale: number;
}

/** Whether a viewport places its content itself, or shows it the way someone left it. */
export enum EPanZoomMode {
  /** Centred and scaled to whatever the viewport currently is, so it stays fitted as the viewport changes. */
  FIT = "fit",
  /** A camera someone set by panning or zooming, kept until they fit or open something else. */
  MANUAL = "manual",
}

export type IPanZoomState =
  | { mode: EPanZoomMode.FIT }
  | {
      mode: EPanZoomMode.MANUAL;
      camera: IPanZoomCamera;
    };

/** What a viewport shows before anyone has moved it, and what the fit control puts it back to. */
export const PAN_ZOOM_FIT: IPanZoomState = { mode: EPanZoomMode.FIT };

/** Where the content goes, as the css transform `translate(offset) scale(scale)` with an origin of `0 0`. */
export interface IPanZoomTransform {
  scale: number;
  offsetX: number;
  offsetY: number;
}

/** What an unmeasured viewport places its content by: nothing is known yet, so nothing is moved. */
const PAN_ZOOM_UNPLACED: IPanZoomTransform = { scale: 1, offsetX: 0, offsetY: 0 };

export const PAN_ZOOM_MINIMUM_SCALE: number = 0.1;
export const PAN_ZOOM_MAXIMUM_SCALE: number = 32;

/** One wheel notch, as a multiplier rather than an addend so each step feels the same at any zoom. */
const WHEEL_STEP: number = 1.2;

/**
 * Clamps a scale to the supported viewport range.
 *
 * @param scale - Desired scale, where 1 is one content pixel per viewport pixel.
 * @returns The scale, bounded by the minimum and maximum this module allows.
 */
export function clampScale(scale: number): number {
  return Math.min(PAN_ZOOM_MAXIMUM_SCALE, Math.max(PAN_ZOOM_MINIMUM_SCALE, scale));
}

/**
 * Puts a camera under someone's control, at wherever it currently looks.
 *
 * @param camera - Camera the gesture produced.
 * @returns The state a viewport keeps until it is fitted again.
 */
export function toManualPanZoom(camera: IPanZoomCamera): IPanZoomState {
  return { mode: EPanZoomMode.MANUAL, camera };
}

/**
 * The camera that centres content in a viewport and scales it to fit.
 *
 * Never enlarges: something smaller than the viewport is shown at its own size, because blowing a 16px icon up to fill
 * a pane tells you less about it than seeing how small it really is.
 *
 * @param content - Content size in its own pixels.
 * @param viewport - Viewport size in viewport pixels.
 * @returns The fitted camera, at the content's centre.
 */
export function toFitCamera(content: IPanZoomSize, viewport: IPanZoomSize): IPanZoomCamera {
  const center: IPanZoomPoint = { x: content.width / 2, y: content.height / 2 };

  if (!hasArea(content) || !hasArea(viewport)) {
    return { center, scale: 1 };
  }

  return {
    center,
    scale: clampScale(Math.min(1, viewport.width / content.width, viewport.height / content.height)),
  };
}

/**
 * The camera a state is looking through, working out the fit if it has none of its own.
 *
 * @param state - What the viewport is showing.
 * @param content - Content size in its own pixels.
 * @param viewport - Viewport size in viewport pixels.
 * @returns The camera, ready for a gesture to move.
 */
export function resolvePanZoomCamera(
  state: IPanZoomState,
  content: IPanZoomSize,
  viewport: IPanZoomSize
): IPanZoomCamera {
  return state.mode === EPanZoomMode.MANUAL ? state.camera : toFitCamera(content, viewport);
}

/**
 * Where the content sits for a viewport of this size.
 *
 * @param state - What the viewport is showing.
 * @param content - Content size in its own pixels.
 * @param viewport - Viewport size in viewport pixels.
 * @returns The transform placing the content, or the identity while either size is still unknown.
 */
export function toPanZoomTransform(
  state: IPanZoomState,
  content: IPanZoomSize,
  viewport: IPanZoomSize
): IPanZoomTransform {
  if (!hasArea(content) || !hasArea(viewport)) {
    return PAN_ZOOM_UNPLACED;
  }

  const camera: IPanZoomCamera = resolvePanZoomCamera(state, content, viewport);

  // The camera's centre lands on the viewport's, and the transform origin is the content's top left corner.
  return {
    scale: camera.scale,
    offsetX: viewport.width / 2 - camera.center.x * camera.scale,
    offsetY: viewport.height / 2 - camera.center.y * camera.scale,
  };
}

/**
 * What the content-space coordinate under a viewport coordinate is.
 *
 * @param camera - Camera the viewport is looking through.
 * @param viewport - Viewport size in viewport pixels.
 * @param point - Point in viewport coordinates, measured from the viewport's top left corner.
 * @returns The content-space point beneath it.
 */
export function toContentPoint(camera: IPanZoomCamera, viewport: IPanZoomSize, point: IPanZoomPoint): IPanZoomPoint {
  return {
    x: camera.center.x + (point.x - viewport.width / 2) / camera.scale,
    y: camera.center.y + (point.y - viewport.height / 2) / camera.scale,
  };
}

/**
 * Shifts the content without changing its scale.
 *
 * @param camera - Camera the viewport is looking through.
 * @param deltaX - Horizontal movement in viewport pixels.
 * @param deltaY - Vertical movement in viewport pixels.
 * @returns The camera moved against the drag, since dragging the content right looks left.
 */
export function panBy(camera: IPanZoomCamera, deltaX: number, deltaY: number): IPanZoomCamera {
  return {
    center: { x: camera.center.x - deltaX / camera.scale, y: camera.center.y - deltaY / camera.scale },
    scale: camera.scale,
  };
}

/**
 * Rescales while keeping one content point where it is on screen.
 *
 * Zooming about the viewport centre instead makes the thing being inspected drift off screen at high magnification,
 * which is when it matters most.
 *
 * @param camera - Camera the viewport is looking through.
 * @param anchor - Content-space point to hold still, from `toContentPoint` for a pointer.
 * @param nextScale - Desired scale, clamped before use.
 * @returns The camera at the new scale, with the anchor over the same pixel.
 */
export function zoomAround(camera: IPanZoomCamera, anchor: IPanZoomPoint, nextScale: number): IPanZoomCamera {
  const scale: number = clampScale(nextScale);
  // The anchor's offset from the centre is fixed in viewport pixels, so in content space it shrinks as the scale grows.
  const ratio: number = camera.scale / scale;

  return {
    center: {
      x: anchor.x + (camera.center.x - anchor.x) * ratio,
      y: anchor.y + (camera.center.y - anchor.y) * ratio,
    },
    scale,
  };
}

/**
 * Applies one wheel zoom step at the pointer.
 *
 * @param camera - Camera the viewport is looking through.
 * @param anchor - Content-space point under the pointer, from `toContentPoint`.
 * @param delta - Wheel delta, where positive scrolls down and so zooms out.
 * @returns The camera one notch away, anchored on the pointer.
 */
export function zoomByWheel(camera: IPanZoomCamera, anchor: IPanZoomPoint, delta: number): IPanZoomCamera {
  return zoomAround(camera, anchor, delta > 0 ? camera.scale / WHEEL_STEP : camera.scale * WHEEL_STEP);
}

/**
 * Whether a size is one anything can be laid out against.
 *
 * @param size - Size to check.
 * @returns Whether both of its dimensions are known and non-zero.
 */
function hasArea(size: IPanZoomSize): boolean {
  return size.width > 0 && size.height > 0;
}
