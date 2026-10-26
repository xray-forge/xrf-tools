import { RenderPageBackdrop } from "@xrf/ipc/types/xrf-renderer";

import { readNativePageBackdrop } from "@/core/render/lib/native/native-page-backdrop";
import { INativePagePlanes, readNativePagePlanes } from "@/core/render/lib/native/native-page-planes";
import { readColorScheme, watchColorScheme } from "@/core/theme/color-scheme";
import { getWashStops, IWashStops } from "@/core/theme/surface";
import { ColorScheme } from "@/core/theme/tokens";

/** An element's inline background, put back as it was when the hole closes. */
interface IClearedBackground {
  element: HTMLElement;
  background: string;
}

/**
 * A hole down to the window through the page, which a native viewport is drawn into: every plane painting under the
 * viewport is told to the renderer, which paints each in its own box instead, and its background is taken off the page.
 *
 * The planes are read at once, so the first layout carries them, but the backgrounds come off only when the hole opens,
 * once the renderer has drawn the window: until then the window is bare, black while the renderer starts its device.
 * The planes' colours follow the colour scheme, so the page is read again whenever it changes.
 */
export class NativeViewportHole {
  private readonly stopWatching: () => void;
  private readonly cleared: Array<IClearedBackground> = [];
  private page: INativePagePlanes;
  private stops: IWashStops;
  private isOpen: boolean = false;

  public constructor(private readonly element: HTMLElement) {
    this.page = readNativePagePlanes(element);
    this.stops = getWashStops(readColorScheme());
    this.stopWatching = watchColorScheme(this.onSchemeChange);
  }

  /** Takes the planes' backgrounds off the page, the renderer painting them now; again does nothing. */
  public open(): void {
    if (!this.isOpen) {
      this.isOpen = true;
      this.clear();
    }
  }

  public dispose(): void {
    this.stopWatching();
    this.restore();
  }

  /**
   * @param scale - Device pixels a CSS pixel.
   * @returns What the page shows under the viewport where it lays out now.
   */
  public getBackdrop(scale: number): RenderPageBackdrop {
    return readNativePageBackdrop(this.page, this.stops, scale);
  }

  private clear(): void {
    for (const { element } of this.page.planes) {
      this.cleared.push({ background: element.style.background, element });
      element.style.background = "transparent";
    }
  }

  private restore(): void {
    for (const { element, background } of this.cleared.splice(0).reverse()) {
      element.style.background = background;
    }
  }

  /** Reads the page with its backgrounds back, before it paints, so they show for no frame. */
  private readonly onSchemeChange = (scheme: ColorScheme): void => {
    this.restore();
    this.page = readNativePagePlanes(this.element);
    this.stops = getWashStops(scheme);

    if (this.isOpen) {
      this.clear();
    }
  };
}
