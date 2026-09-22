import { Nullable } from "@xrf/types";

import { IRenderTarget } from "#/frame/render-target";

/** How big a transferred canvas is, which only the side that still has a document can measure. */
export interface IOffscreenRenderSize {
  width: number;
  height: number;
  pixelRatio: number;
}

/**
 * A canvas transferred away from the page.
 */
export class OffscreenRenderTarget implements IRenderTarget {
  public readonly canvas: OffscreenCanvas;
  /** Nothing here has a style: the element the canvas came from is on a thread this one cannot reach. */
  public readonly isStyled: boolean = false;

  private size: IOffscreenRenderSize;
  private onResized: Nullable<() => void> = null;

  public constructor(canvas: OffscreenCanvas, size: IOffscreenRenderSize) {
    this.canvas = canvas;
    this.size = size;
  }

  public get pixelRatio(): number {
    return this.size.pixelRatio;
  }

  public get width(): number {
    return this.size.width;
  }

  public get height(): number {
    return this.size.height;
  }

  /**
   * Takes the size the page measured.
   *
   * @param size - What the element the canvas came from now is.
   */
  public resize(size: IOffscreenRenderSize): void {
    this.size = size;

    this.onResized?.();
  }

  public observe(onResized: () => void): () => void {
    this.onResized = onResized;

    return (): void => {
      this.onResized = null;
    };
  }

  /** Nothing of the page is held here, so there is nothing of it to give back. */
  public dispose(): void {
    this.onResized = null;
  }
}
