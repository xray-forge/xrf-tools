import { Nullable } from "@xrf/types";
import { CanvasTarget, WebGPURenderer } from "three/webgpu";

import { IRendererViewSize } from "#/contract/renderer-view-size";

/**
 * The page canvas frames are shown on, handed to this thread, and the size the page last measured it at.
 */
export class RendererView {
  /**
   * A size a device pixel across at least: a hidden element measures nothing, a ratio under one floors a css pixel to
   * none, and nothing allocates an empty target.
   */
  private static toDrawnSize(size: IRendererViewSize): IRendererViewSize {
    const { width, height, pixelRatio } = size;

    return {
      ...size,
      height: RendererView.toDrawnSide(height, pixelRatio),
      width: RendererView.toDrawnSide(width, pixelRatio),
    };
  }

  /** A css side three draws at least one device pixel of: it floors the side times the ratio. */
  private static toDrawnSide(side: number, pixelRatio: number): number {
    const drawn: number = Math.max(1, side, pixelRatio > 0 ? Math.ceil(1 / pixelRatio) : 1);

    // Rounding can leave the least side a hair short of its pixel.
    return pixelRatio > 0 && Math.floor(drawn * pixelRatio) < 1 ? drawn + 1 : drawn;
  }

  public readonly canvas: OffscreenCanvas;

  private currentSize: IRendererViewSize;
  /** Made once a device shows the view; three draws on whichever canvas target is current. */
  private target: Nullable<CanvasTarget> = null;
  private isResizePending: boolean = true;

  public constructor(canvas: OffscreenCanvas, size: IRendererViewSize) {
    this.canvas = canvas;
    this.currentSize = RendererView.toDrawnSize(size);
  }

  public get size(): IRendererViewSize {
    return this.currentSize;
  }

  /**
   * @param size - What the element the canvas came from now is.
   */
  public resize(size: IRendererViewSize): void {
    this.currentSize = RendererView.toDrawnSize(size);
    this.isResizePending = true;
  }

  /**
   * @returns Whether the size changed since last asked, which the frame about to be drawn has to take.
   */
  public takeResize(): boolean {
    const isPending: boolean = this.isResizePending;

    this.isResizePending = false;

    return isPending;
  }

  /**
   * Makes the canvas what a device draws on.
   *
   * @param renderer - The device's renderer.
   */
  public show(renderer: WebGPURenderer): void {
    if (!this.target) {
      this.target = new CanvasTarget(this.canvas);
      this.isResizePending = true;
      renderer.setCanvasTarget(this.target);
    }
  }

  /** Lets go of what the device held of the canvas. */
  public hide(): void {
    this.target?.dispose();
    this.target = null;
  }
}
