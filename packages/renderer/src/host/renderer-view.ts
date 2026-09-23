import { Nullable } from "@xrf/types";
import { CanvasTarget, WebGPURenderer } from "three/webgpu";

import { IRendererViewSize } from "#/contract/renderer-view-size";

/**
 * The page canvas frames are shown on, handed to this thread, and the size the page last measured it at.
 */
export class RendererView {
  public readonly canvas: OffscreenCanvas;

  private currentSize: IRendererViewSize;
  /** Made once a device shows the view; three draws on whichever canvas target is current. */
  private target: Nullable<CanvasTarget> = null;
  private isResizePending: boolean = true;

  public constructor(canvas: OffscreenCanvas, size: IRendererViewSize) {
    this.canvas = canvas;
    this.currentSize = size;
  }

  public get size(): IRendererViewSize {
    return this.currentSize;
  }

  /**
   * @param size - What the element the canvas came from now is.
   */
  public resize(size: IRendererViewSize): void {
    this.currentSize = size;
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

  /** Lets go of what the device held of the canvas, which a later device can show again. */
  public hide(): void {
    this.target?.dispose();
    this.target = null;
  }
}
