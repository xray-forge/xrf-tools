import { Nullable } from "@xrf/types";

import { ERenderInputKind, RenderColor, RenderInputEvent, RenderViewportLayout } from "@/core/ipc/types/xrf-renderer";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";

/** What the pointer shows while it drags a scene, as the WebGPU viewports show it. */
const DRAG_CURSOR: string = "grabbing";

/** An element's background, put back as it was when the hole closes. */
interface IClearedBackground {
  element: HTMLElement;
  background: string;
}

/**
 * The page's side of a native viewport: an element the renderer draws under.
 *
 * It opens a hole down to the window by taking the backgrounds off the element and every ancestor, reports where the
 * element is whenever that changes, and forwards what the pointer and keys do over it. Pointer moves are merged into
 * one an animation frame, since a camera reads only how far the pointer went.
 */
export class NativeViewportTarget {
  private readonly cleared: Array<IClearedBackground> = [];
  /** The page's colour around the hole, which the renderer clears to where layout has not caught up. */
  private readonly clear: RenderColor;
  private readonly hadTabIndex: boolean;
  private frame: number = 0;
  /** The layout last sent, as compared. */
  private sent: string = "";
  private rect: DOMRect;
  /** The latest pointer move not yet sent. */
  private moved: Nullable<RenderInputEvent> = null;
  /** The cursor the element had before a drag took it, while one drags. */
  private resting: Nullable<string> = null;

  public constructor(
    private readonly element: HTMLElement,
    private readonly viewport: NativeViewport
  ) {
    this.hadTabIndex = element.hasAttribute("tabindex");

    // Focusable, so its keys reach it; drawn over by nothing, so no ring shows on a click.
    if (!this.hadTabIndex) {
      element.tabIndex = 0;
    }

    element.style.outline = "none";
    this.clear = this.openHole();
    this.rect = element.getBoundingClientRect();

    for (const [type, listener] of Object.entries(this.listeners)) {
      element.addEventListener(type, listener as EventListener, { passive: false });
    }

    this.poll();
  }

  public dispose(): void {
    cancelAnimationFrame(this.frame);

    for (const [type, listener] of Object.entries(this.listeners)) {
      this.element.removeEventListener(type, listener as EventListener);
    }

    for (const { element, background } of this.cleared.splice(0).reverse()) {
      element.style.background = background;
    }

    this.endDrag();
    this.element.style.outline = "";

    if (!this.hadTabIndex) {
      this.element.removeAttribute("tabindex");
    }
  }

  /**
   * Takes the background off the element and each ancestor that paints one, down to the document.
   *
   * @returns The colour the nearest of them painted, which is what the page showed around the viewport.
   */
  private openHole(): RenderColor {
    let clear: Nullable<RenderColor> = null;

    for (let element: Nullable<HTMLElement> = this.element; element; element = element.parentElement) {
      const style: CSSStyleDeclaration = getComputedStyle(element);
      const color: Nullable<RenderColor> = toOpaqueColor(style.backgroundColor);

      if (color || style.backgroundImage !== "none") {
        clear ??= color;
        this.cleared.push({ background: element.style.background, element });
        element.style.background = "transparent";
      }
    }

    return clear ?? { b: 0, g: 0, r: 0 };
  }

  /** Reports the layout when it moved, and sends the merged pointer move, once an animation frame. */
  private readonly poll = (): void => {
    this.frame = requestAnimationFrame(this.poll);
    this.rect = this.element.getBoundingClientRect();
    this.flushMove();

    const scale: number = window.devicePixelRatio;
    const left: number = Math.round(this.rect.left * scale);
    const top: number = Math.round(this.rect.top * scale);
    const layout: RenderViewportLayout = {
      clear: this.clear,
      rect: {
        height: Math.max(0, Math.round(this.rect.bottom * scale) - top),
        width: Math.max(0, Math.round(this.rect.right * scale) - left),
        x: left,
        y: top,
      },
      scale,
    };
    const key: string = JSON.stringify(layout);

    if (key !== this.sent) {
      this.sent = key;
      this.viewport.setLayout(layout);
    }
  };

  private flushMove(): void {
    if (this.moved) {
      this.viewport.sendInput(this.moved);
      this.moved = null;
    }
  }

  private send(kind: ERenderInputKind, event: Event): void {
    // Whatever moved before this gesture goes first, so a release never arrives ahead of the move before it.
    this.flushMove();
    this.viewport.sendInput(this.toInput(kind, event));
  }

  private toInput(kind: ERenderInputKind, event: Event): RenderInputEvent {
    const pointer: Partial<PointerEvent> = event as PointerEvent;
    const wheel: Partial<WheelEvent> = event as WheelEvent;
    const key: Partial<KeyboardEvent> = event as KeyboardEvent;

    return {
      altKey: key.altKey ?? false,
      button: pointer.button ?? 0,
      buttons: pointer.buttons ?? 0,
      code: key.code ?? "",
      ctrlKey: key.ctrlKey ?? false,
      deltaMode: wheel.deltaMode ?? 0,
      deltaX: wheel.deltaX ?? 0,
      deltaY: wheel.deltaY ?? 0,
      isPrimary: pointer.isPrimary ?? false,
      kind,
      metaKey: key.metaKey ?? false,
      pointerId: pointer.pointerId ?? 0,
      shiftKey: key.shiftKey ?? false,
      x: (pointer.clientX ?? 0) - this.rect.left,
      y: (pointer.clientY ?? 0) - this.rect.top,
    };
  }

  private startDrag(): void {
    this.resting ??= this.element.style.cursor;
    this.element.style.cursor = DRAG_CURSOR;
  }

  private endDrag(): void {
    if (this.resting !== null) {
      this.element.style.cursor = this.resting;
      this.resting = null;
    }
  }

  private readonly listeners: Readonly<Record<string, (event: Event) => void>> = {
    blur: (event: Event): void => {
      this.endDrag();
      this.send(ERenderInputKind.BLUR, event);
    },
    contextmenu: (event: Event): void => {
      event.preventDefault();
      this.send(ERenderInputKind.CONTEXT_MENU, event);
    },
    keydown: (event: Event): void => {
      // A held key repeats; the renderer already holds it.
      if (!(event as KeyboardEvent).repeat) {
        this.send(ERenderInputKind.KEY_DOWN, event);
      }
    },
    keyup: (event: Event): void => this.send(ERenderInputKind.KEY_UP, event),
    pointercancel: (event: Event): void => {
      this.endDrag();
      this.send(ERenderInputKind.POINTER_CANCEL, event);
    },
    pointerdown: (event: Event): void => {
      const pointer: PointerEvent = event as PointerEvent;

      this.element.focus({ preventScroll: true });
      this.element.setPointerCapture?.(pointer.pointerId);

      if (pointer.isPrimary && pointer.button === 0) {
        this.startDrag();
      }

      this.send(ERenderInputKind.POINTER_DOWN, event);
    },
    pointermove: (event: Event): void => {
      this.moved = this.toInput(ERenderInputKind.POINTER_MOVE, event);
    },
    pointerup: (event: Event): void => {
      this.endDrag();
      this.send(ERenderInputKind.POINTER_UP, event);
    },
    wheel: (event: Event): void => {
      event.preventDefault();
      this.send(ERenderInputKind.WHEEL, event);
    },
  };
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
