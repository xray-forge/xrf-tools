import { Nullable } from "@xrf/types";

import { RenderColor, RenderPageBackdrop } from "@/core/ipc/types/xrf-renderer";
import { getWashStops, IWashStops } from "@/core/theme/surface";
import { ColorScheme } from "@/core/theme/tokens";

/** The attribute the theme names the colour scheme shown by, on the document element. */
const SCHEME_ATTRIBUTE: string = "data-color-scheme";

/** An element's background, put back as it was when the hole closes. */
interface IClearedBackground {
  element: HTMLElement;
  background: string;
}

/**
 * A hole down to the window through the page, which a native viewport is drawn into: the backgrounds taken off an
 * element and every ancestor that paints one, and what they painted told to the renderer, which paints it under the
 * viewport instead.
 *
 * The page's colours follow its colour scheme, so the hole is opened again whenever the scheme changes, reading what
 * each ancestor paints in the scheme now shown.
 */
export class NativeViewportHole {
  private readonly cleared: Array<IClearedBackground> = [];
  private readonly observer: MutationObserver;
  /** The colour the nearest painted ancestor showed around the viewport. */
  private color: RenderColor = { b: 0, g: 0, r: 0 };
  /** The ancestor whose wash shows around the viewport, or null where the nearest paints none. */
  private washed: Nullable<HTMLElement> = null;
  private stops: IWashStops;

  public constructor(private readonly element: HTMLElement) {
    this.stops = getWashStops(readColorScheme());
    this.open();
    this.observer = new MutationObserver(() => this.reopen());
    this.observer.observe(document.documentElement, { attributeFilter: [SCHEME_ATTRIBUTE], attributes: true });
  }

  public dispose(): void {
    this.observer.disconnect();
    this.close();
  }

  /**
   * @param scale - Device pixels a CSS pixel.
   * @returns What the page shows through the hole now, the wash laid over the box of the ancestor painting it.
   */
  public getBackdrop(scale: number): RenderPageBackdrop {
    if (!this.washed) {
      return { color: this.color, wash: null };
    }

    const box: DOMRect = this.washed.getBoundingClientRect();
    const left: number = Math.round(box.left * scale);
    const top: number = Math.round(box.top * scale);

    return {
      color: this.color,
      wash: {
        angle: this.stops.angle,
        from: this.stops.first,
        rect: {
          height: Math.max(0, Math.round(box.bottom * scale) - top),
          width: Math.max(0, Math.round(box.right * scale) - left),
          x: left,
          y: top,
        },
        to: this.stops.last,
      },
    };
  }

  /** Takes the background off the element and each ancestor that paints one, reading what the nearest painted. */
  private open(): void {
    let isFound: boolean = false;

    for (let element: Nullable<HTMLElement> = this.element; element; element = element.parentElement) {
      const style: CSSStyleDeclaration = getComputedStyle(element);
      const color: Nullable<RenderColor> = toOpaqueColor(style.backgroundColor);
      const isWashed: boolean = isPaintedImage(style.backgroundImage);

      if (color || isWashed) {
        if (!isFound) {
          isFound = true;
          this.color = color ?? this.color;
          this.washed = isWashed ? element : null;
        }

        this.cleared.push({ background: element.style.background, element });
        element.style.background = "transparent";
      }
    }
  }

  private close(): void {
    for (const { element, background } of this.cleared.splice(0).reverse()) {
      element.style.background = background;
    }
  }

  /** Reads the page again in the scheme it shows now, before it paints: the backgrounds are back for no frame. */
  private reopen(): void {
    this.close();
    this.color = { b: 0, g: 0, r: 0 };
    this.washed = null;
    this.stops = getWashStops(readColorScheme());
    this.open();
  }
}

/** Whether a computed `background-image` paints anything: `none`, or nothing at all where it is not computed. */
function isPaintedImage(image: string): boolean {
  return image !== "" && image !== "none";
}

function readColorScheme(): ColorScheme {
  return document.documentElement.getAttribute(SCHEME_ATTRIBUTE) === "light" ? "light" : "dark";
}

/**
 * @param css - A computed colour, `rgb(...)` or `rgba(...)`.
 * @returns The colour, or null for one that paints nothing.
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
