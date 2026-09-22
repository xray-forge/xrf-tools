import { IOffscreenRenderSize } from "@xrf/renderer";

import { IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { noop } from "@/lib/callbacks/noop";

/** What a scene binds its controls to: the canvas on a page, or something answering for it on a worker. */
export type TRenderInputElement = HTMLElement | RenderProxyElement;

/** Told whatever the controls wrote on the element's cursor, for the side that has a cursor. */
export type TRenderCursorSink = (cursor: string) => void;

/** The event as it crossed, plus the two calls a control makes on one. */
export interface IRenderProxyEvent extends IRenderInputEvent {
  preventDefault(): void;
  stopPropagation(): void;
}

/** What a listener is given. */
type TProxyListener = (event: IRenderProxyEvent) => void;

/**
 * Stands in for the canvas on the thread that cannot reach it.
 */
export class RenderProxyElement {
  /** What the controls wrote, watched so the side with a real canvas can show it. */
  public readonly style: { cursor: string; touchAction: string };

  private readonly listeners: Map<string, Set<TProxyListener>> = new Map();
  private size: IOffscreenRenderSize;
  private cursor: string = "";

  public constructor(size: IOffscreenRenderSize, onCursor: TRenderCursorSink) {
    this.size = size;
    this.style = {
      cursor: "",
      touchAction: "",
    };

    // Watched rather than copied, because three writes `grab`, `grabbing` and `auto` on the element itself and
    // a scene that reported its own would be saying it twice.
    Object.defineProperty(this.style, "cursor", {
      get: (): string => this.cursor,
      set: (cursor: string): void => {
        this.cursor = cursor;

        onCursor(cursor);
      },
    });
  }

  public get ownerDocument(): RenderProxyElement {
    return this;
  }

  public get clientWidth(): number {
    return this.size.width;
  }

  public get clientHeight(): number {
    return this.size.height;
  }

  /** @returns Itself, which is as far as anything here can look for a document. */
  public getRootNode(): RenderProxyElement {
    return this;
  }

  /** @returns Where the canvas is, which is at the origin of a thread that has nothing else in it. */
  public getBoundingClientRect(): DOMRect {
    const { width, height } = this.size;

    return { bottom: height, height, left: 0, right: width, top: 0, width, x: 0, y: 0 } as DOMRect;
  }

  /**
   * Takes the size the page measured.
   *
   * @param size - What the element the canvas came from now is.
   */
  public resize(size: IOffscreenRenderSize): void {
    this.size = size;
  }

  /**
   * @param type - Gesture to hear about.
   * @param listener - Told when one arrives.
   */
  public addEventListener(type: string, listener: TProxyListener): void {
    const listeners: Set<TProxyListener> = this.listeners.get(type) ?? new Set();

    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  /**
   * @param type - Gesture to stop hearing about.
   * @param listener - The one to stop telling.
   */
  public removeEventListener(type: string, listener: TProxyListener): void {
    this.listeners.get(type)?.delete(listener);
  }

  /**
   * Hands one gesture to whatever is listening for it.
   *
   * @param event - The gesture, as it crossed.
   */
  public dispatch(event: IRenderInputEvent): void {
    const listeners: Set<TProxyListener> = this.listeners.get(event.type) ?? new Set();

    const dispatched: IRenderProxyEvent = { ...event, preventDefault: noop, stopPropagation: noop };

    // Copied, because a control that lets go of a pointer stops listening from inside the call that tells it.
    for (const listener of [...listeners]) {
      listener(dispatched);
    }
  }

  /** Nothing on this side owns the pointer: the events arrive wherever the page sends them. */
  public setPointerCapture(): void {
    return;
  }

  public releasePointerCapture(): void {
    return;
  }

  public hasPointerCapture(): boolean {
    return false;
  }
}
