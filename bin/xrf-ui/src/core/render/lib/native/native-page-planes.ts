import { Nullable } from "@xrf/types";

import { RenderColor } from "@/core/ipc/types/xrf-renderer";
import {
  ICssCorners,
  ICssEdges,
  isClipping,
  readBorderRadii,
  readBorderWidths,
  toPaddingRadii,
} from "@/lib/dom/css-box";

/** An ancestor clipping what it holds to its padding box. */
export interface INativePageClip {
  element: HTMLElement;
  borders: ICssEdges;
  /** Its padding box's corner radii. */
  radii: ICssCorners;
}

/** An element painting a background under a viewport, which the renderer paints in its place. */
export interface INativePagePlane {
  element: HTMLElement;
  /** Its colour; one painting only an image shows the colour of the plane outside it. */
  color: RenderColor;
  /** Whether it paints the theme's wash. */
  isWashed: boolean;
  /** Whether its wash is laid over the page's viewport, as `background-attachment: fixed` lays it, or over its box. */
  isWashFixed: boolean;
  radii: ICssCorners;
  /** The ancestors clipping it, each shared with the planes inside it. */
  clips: ReadonlyArray<INativePageClip>;
}

/** What paints under a viewport: its element's painted planes, outermost first, and every ancestor clipping one. */
export interface INativePagePlanes {
  planes: ReadonlyArray<INativePagePlane>;
  clips: ReadonlyArray<INativePageClip>;
}

/**
 * Reads what the viewport's element and each of its ancestors paints, outermost first: a plane is clipped by the clips
 * read before it, and one painting only an image shows the colour read before it, black outside every colour.
 *
 * @param element - The viewport's element.
 * @returns Its painted planes and the ancestors clipping them, from their computed styles.
 */
export function readNativePagePlanes(element: HTMLElement): INativePagePlanes {
  const planes: Array<INativePagePlane> = [];
  const clips: Array<INativePageClip> = [];
  let outer: RenderColor = { b: 0, g: 0, r: 0 };

  for (const it of listAncestry(element).reverse()) {
    const style: CSSStyleDeclaration = getComputedStyle(it);
    const color: Nullable<RenderColor> = toOpaqueColor(style.backgroundColor);
    const isWashed: boolean = isPaintedImage(style.backgroundImage);

    if (color || isWashed) {
      outer = color ?? outer;
      planes.push({
        clips: [...clips],
        color: outer,
        element: it,
        isWashFixed: style.backgroundAttachment === "fixed",
        isWashed,
        radii: readBorderRadii(style),
      });
    }

    // An element clips what it holds, never its own background.
    if (isClipping(style)) {
      const borders: ICssEdges = readBorderWidths(style);

      clips.push({ borders, element: it, radii: toPaddingRadii(readBorderRadii(style), borders) });
    }
  }

  return { clips, planes };
}

/**
 * @param css - A computed colour, `rgb(...)` or `rgba(...)`.
 * @returns The colour, its alpha dropped, or null for one that paints nothing.
 */
export function toOpaqueColor(css: string): Nullable<RenderColor> {
  const channels: Nullable<RegExpMatchArray> = css.match(/rgba?\(([^)]+)\)/);

  if (!channels) {
    return null;
  }

  const [r = 0, g = 0, b = 0, a = 1] = channels[1]
    .split(/[\s,/]+/)
    .filter(Boolean)
    .map((part: string) => Number.parseFloat(part));

  return a > 0 ? { b: Math.round(b), g: Math.round(g), r: Math.round(r) } : null;
}

/** The element and its ancestors, nearest first. */
function listAncestry(element: HTMLElement): Array<HTMLElement> {
  const ancestry: Array<HTMLElement> = [];

  for (let it: Nullable<HTMLElement> = element; it; it = it.parentElement) {
    ancestry.push(it);
  }

  return ancestry;
}

/** Whether a computed `background-image` paints anything: `none`, or nothing at all where it is not computed. */
function isPaintedImage(image: string): boolean {
  return image !== "" && image !== "none";
}
