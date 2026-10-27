import { RenderPageBackdrop, RenderPagePlane } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { IWashStops } from "@xrf/ui/theme/surface";

import { INativePageClip, INativePagePlane, INativePagePlanes } from "@/core/render/lib/native/native-page-planes";
import { toNativeRect } from "@/core/render/lib/native/native-rect";
import {
  getSharedCorners,
  ICssBox,
  ICssCorners,
  intersectBoxes,
  isCovering,
  maxCorners,
  toPaddingBox,
} from "@/lib/dom/css-box";

/** A clip where the page lays it out now: its padding box and that box's radii. */
interface IShownClip {
  box: ICssBox;
  radii: ICssCorners;
}

/** A plane where the page lays it out now. */
interface IShownPlane {
  plane: INativePagePlane;
  /** Its border box, which an unfixed wash is laid over. */
  own: ICssBox;
  /** What of it its clips show. */
  box: ICssBox;
  /** Its own radii, and a clip's where it fills that clip's rounded corner. */
  radii: ICssCorners;
}

/**
 * Measures what the page shows under a viewport where it lays out now, starting from the innermost plane filling the
 * page, which the renderer fills the window with.
 *
 * @param page - The planes and clips painting under the viewport.
 * @param stops - The theme's wash in the scheme shown.
 * @param scale - Device pixels a CSS pixel.
 * @returns Each plane shown, outermost first, and the wash over the box of the innermost washed one.
 */
export function readNativePageBackdrop(page: INativePagePlanes, stops: IWashStops, scale: number): RenderPageBackdrop {
  const view: ICssBox = readViewBox();
  // Planes share their clips, so each clip is measured once.
  const clips: Map<INativePageClip, IShownClip> = new Map(
    page.clips.map((clip: INativePageClip) => [clip, measureClip(clip)])
  );
  const shown: Array<IShownPlane> = page.planes.map((plane: INativePagePlane) => measurePlane(plane, clips));
  const drawn: Array<IShownPlane> = shown.slice(findBase(shown, view));
  const washed: Nullable<IShownPlane> = drawn.filter((it: IShownPlane) => it.plane.isWashed).pop() ?? null;

  return {
    planes: drawn.map((it: IShownPlane) => toRenderPlane(it, scale)),
    wash: washed
      ? {
          angle: stops.angle,
          from: stops.first,
          rect: toNativeRect(washed.plane.isWashFixed ? view : washed.own, scale),
          to: stops.last,
        }
      : null,
  };
}

/** The innermost plane filling the page, which hides every one outside it; the outermost where none does. */
function findBase(shown: Array<IShownPlane>, view: ICssBox): number {
  for (let index: number = shown.length - 1; index > 0; index -= 1) {
    if (isCovering(shown[index].box, view)) {
      return index;
    }
  }

  return 0;
}

function measureClip(clip: INativePageClip): IShownClip {
  return { box: toPaddingBox(clip.element.getBoundingClientRect(), clip.borders), radii: clip.radii };
}

/** Cuts a plane to its clips, then rounds the corners it shares with a rounded clip, whichever clip cut it there. */
function measurePlane(plane: INativePagePlane, measured: Map<INativePageClip, IShownClip>): IShownPlane {
  const own: ICssBox = plane.element.getBoundingClientRect();
  // A plane's clips are the page's own, every one measured.
  const clips: Array<IShownClip> = plane.clips.map((clip: INativePageClip) => measured.get(clip)!);
  const box: ICssBox = clips.reduce((it: ICssBox, clip: IShownClip) => intersectBoxes(it, clip.box), own);
  const radii: ICssCorners = clips.reduce(
    (it: ICssCorners, clip: IShownClip) => maxCorners(it, getSharedCorners(box, clip.box, clip.radii)),
    plane.radii
  );

  return { box, own, plane, radii };
}

/** The page's viewport, which a fixed background is laid over. */
function readViewBox(): ICssBox {
  const { clientHeight, clientWidth } = document.documentElement;

  return { bottom: clientHeight, left: 0, right: clientWidth, top: 0 };
}

function toRenderPlane({ plane, box, radii }: IShownPlane, scale: number): RenderPagePlane {
  return {
    color: plane.color,
    isWashed: plane.isWashed,
    radii: [radii.topLeft * scale, radii.topRight * scale, radii.bottomRight * scale, radii.bottomLeft * scale],
    rect: toNativeRect(box, scale),
  };
}
