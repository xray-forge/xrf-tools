import { IRenderTarget } from "@/core/render/lib/frame/render-target";
import { bindSelectionReset } from "@/lib/dom/selection";
import { Nullable } from "@/lib/types/general";

/**
 * A canvas filling an element of the page.
 */
export class DomRenderTarget implements IRenderTarget {
  public readonly canvas: HTMLCanvasElement = document.createElement("canvas");

  private readonly container: HTMLElement;
  private readonly resizeObserver: ResizeObserver;
  /** Stops the canvas clearing the window's text selection, called when the target goes. */
  private readonly unbindSelectionReset: () => void;

  private onResized: Nullable<() => void> = null;

  public constructor(container: HTMLElement) {
    this.container = container;
    this.canvas.style.display = "block";
    // Focusable, because a scene reading the keyboard needs somewhere for the focus to be.
    this.canvas.tabIndex = 0;

    container.appendChild(this.canvas);

    this.resizeObserver = new ResizeObserver(() => this.onResized?.());
    this.resizeObserver.observe(container);

    // Bound here rather than per scene: pressing a canvas is the one gesture every preview shares, and a
    // selection left standing behind one belongs to no scene in particular.
    this.unbindSelectionReset = bindSelectionReset(this.canvas);
  }

  public get pixelRatio(): number {
    return window.devicePixelRatio;
  }

  public get width(): number {
    return this.container.clientWidth;
  }

  public get height(): number {
    return this.container.clientHeight;
  }

  public observe(onResized: () => void): () => void {
    this.onResized = onResized;

    return (): void => {
      this.onResized = null;
    };
  }

  public dispose(): void {
    this.resizeObserver.disconnect();
    this.unbindSelectionReset();
    this.canvas.remove();
  }
}
