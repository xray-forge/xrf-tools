import { Nullable } from "@xrf/types";

import { RenderViewportLayout } from "@/core/ipc/types/xrf-renderer";
import { EWorldInputKind, WorldInputEvent } from "@/core/ipc/types/xrf-world";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { NativeViewportHole } from "@/core/render/lib/native/native-viewport-hole";
import { clearWindowTextSelection } from "@/lib/dom/selection";

/** What the pointer shows while it drags a scene. */
const DRAG_CURSOR: string = "grabbing";

/**
 * The page's side of a native viewport: an element the renderer draws under.
 *
 * It opens a hole down to the window through the page (`NativeViewportHole`), reports where the element is and what the
 * page shows around it whenever that changes, and forwards what the pointer and keys do over it. Pointer moves are merged into
 * one an animation frame, since a camera reads only how far the pointer went.
 */
export class NativeViewportTarget {
  private readonly hole: NativeViewportHole;
  private readonly hadTabIndex: boolean;
  private frame: number = 0;
  /** The layout last sent, as compared. */
  private sent: string = "";
  private rect: DOMRect;
  /** The latest pointer move not yet sent. */
  private moved: Nullable<WorldInputEvent> = null;
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
    this.hole = new NativeViewportHole(element);
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

    this.hole.dispose();

    this.endDrag();
    this.element.style.outline = "";

    if (!this.hadTabIndex) {
      this.element.removeAttribute("tabindex");
    }
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
      backdrop: this.hole.getBackdrop(scale),
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

  private send(kind: EWorldInputKind, event: Event): void {
    // Whatever moved before this gesture goes first, so a release never arrives ahead of the move before it.
    this.flushMove();
    this.viewport.sendInput(this.toInput(kind, event));
  }

  private toInput(kind: EWorldInputKind, event: Event): WorldInputEvent {
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
      this.send(EWorldInputKind.BLUR, event);
    },
    contextmenu: (event: Event): void => {
      event.preventDefault();
      this.send(EWorldInputKind.CONTEXT_MENU, event);
    },
    keydown: (event: Event): void => {
      // A held key repeats; the renderer already holds it.
      if (!(event as KeyboardEvent).repeat) {
        this.send(EWorldInputKind.KEY_DOWN, event);
      }
    },
    keyup: (event: Event): void => this.send(EWorldInputKind.KEY_UP, event),
    // A capture dropped while a button is held ends the drag here, as a release that never arrives would not.
    lostpointercapture: (event: Event): void => {
      this.endDrag();
      this.send(EWorldInputKind.POINTER_CANCEL, event);
    },
    pointercancel: (event: Event): void => {
      this.endDrag();
      this.send(EWorldInputKind.POINTER_CANCEL, event);
    },
    pointerdown: (event: Event): void => {
      const pointer: PointerEvent = event as PointerEvent;

      // No text is selected or dragged from a press on the scene, which would take the drag that follows from it.
      event.preventDefault();
      clearWindowTextSelection();
      this.element.focus({ preventScroll: true });
      this.element.setPointerCapture?.(pointer.pointerId);

      if (pointer.isPrimary && pointer.button === 0) {
        this.startDrag();
      }

      this.send(EWorldInputKind.POINTER_DOWN, event);
    },
    pointermove: (event: Event): void => {
      this.moved = this.toInput(EWorldInputKind.POINTER_MOVE, event);
    },
    pointerup: (event: Event): void => {
      this.endDrag();
      this.send(EWorldInputKind.POINTER_UP, event);
    },
    wheel: (event: Event): void => {
      event.preventDefault();
      this.send(EWorldInputKind.WHEEL, event);
    },
  };
}
