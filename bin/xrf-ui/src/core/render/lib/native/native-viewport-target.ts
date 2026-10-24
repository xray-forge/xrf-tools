import { RenderViewportLayout } from "@xrf/ipc/types/xrf-renderer";
import {
  EWorldCamera,
  EWorldDragEndReason,
  EWorldInputEvent,
  WorldCamera,
  WorldInputEvent,
  WorldInputModifiers,
} from "@xrf/ipc/types/xrf-world";
import { assertExhaustive, Nullable } from "@xrf/types";

import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { requestNativePointerLock } from "@/core/render/lib/native/native-pointer-lock";
import { toNativeRect } from "@/core/render/lib/native/native-rect";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { ENativeViewportAffordance } from "@/core/render/lib/native/native-viewport-affordance";
import { NativeViewportHole } from "@/core/render/lib/native/native-viewport-hole";
import { canFocusViewportOnHover } from "@/core/render/lib/native/native-viewport-hover-focus";
import { clearWindowTextSelection } from "@/lib/dom/selection";

/** What the pointer shows while it drags a scene, until a lock hides it. */
const DRAG_CURSOR: string = "grabbing";

/** Css pixels a press may travel and still be a click, rather than the drag a camera looks around by. */
const RENDER_CLICK_SLOP: number = 4;

/** Milliseconds within which a second click at the same place is a double click's, which picks nothing again. */
const RENDER_DOUBLE_CLICK_TIME: number = 500;

/** Where a press over the viewport stands between its press and its release. */
enum EPressPhase {
  /** Not yet a click or a drag: it has not travelled past the slop. */
  PENDING,
  /** A drag the world is told about. */
  DRAGGING,
  /** Taken away while the button may still be held, as a drag or before it was one: it hears nothing until let go. */
  CANCELLED,
}

/** Whether a drag holds the pointer locked. */
enum EPressLock {
  NONE,
  /** Asked for and not answered; the drag measures the pointer unlocked meanwhile. */
  REQUESTED,
  LOCKED,
}

/** The one press over the viewport being watched. */
interface IPress {
  pointerId: number;
  button: number;
  modifiers: WorldInputModifiers;
  /** Where it went down, in client css pixels. */
  originX: number;
  originY: number;
  /** Where the pointer was last measured unlocked, in client css pixels. */
  lastX: number;
  lastY: number;
  phase: EPressPhase;
  lock: EPressLock;
}

/** The last click heard, so the second of a double click is not heard as another. */
interface IClick {
  time: number;
  x: number;
  y: number;
}

/**
 * Told where a click on the viewport landed, in css pixels from its top left corner, and whether Alt was held, which
 * picks through glass.
 */
export type TNativeViewportClick = (point: IRenderViewPoint, isThrough: boolean) => void;

/**
 * The page's side of a native viewport: an element the renderer draws under.
 *
 * It opens a hole down to the window through the page (`NativeViewportHole`) once the renderer draws the window, reports
 * where the element is and what the page shows around it whenever that changes, and tells the world what the person
 * does over it. It alone tells a click from a drag: a main-button press let go within the slop is a click, handed to
 * `onClick`; a press travelling past it is a drag, sent as gestures with the pointer locked. Drag motion and hovering
 * merge into one event a frame.
 */
export class NativeViewportTarget {
  private readonly hole: NativeViewportHole;
  private readonly hadTabIndex: boolean;
  private frame: number = 0;
  /** The layout last sent, as compared. */
  private sent: string = "";
  private rect: DOMRect;
  private press: Nullable<IPress> = null;
  private clicked: Nullable<IClick> = null;
  /** Drag motion gathered and not yet sent. */
  private moved: { dx: number; dy: number } = { dx: 0, dy: 0 };
  /** The latest hover not yet sent. */
  private hovered: Nullable<IRenderViewPoint> = null;
  /** The cursor the element had before a drag took it, while one drags. */
  private resting: Nullable<string> = null;

  public constructor(
    private readonly element: HTMLElement,
    private readonly viewport: NativeViewport,
    private readonly onClick: Nullable<TNativeViewportClick> = null
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

    element.ownerDocument.addEventListener("pointerlockchange", this.onPointerLockChange);

    this.poll();
  }

  /** Opens the hole through the page, once the renderer has drawn the window; again does nothing. */
  public openHole(): void {
    this.hole.open();
  }

  public dispose(): void {
    cancelAnimationFrame(this.frame);

    for (const [type, listener] of Object.entries(this.listeners)) {
      this.element.removeEventListener(type, listener as EventListener);
    }

    this.element.ownerDocument.removeEventListener("pointerlockchange", this.onPointerLockChange);
    this.hole.dispose();

    if (this.press) {
      this.endDrag(this.press, EWorldDragEndReason.CANCELLED);
      this.press = null;
    }

    this.element.style.outline = "";

    if (!this.hadTabIndex) {
      this.element.removeAttribute("tabindex");
    }
  }

  /** Reports the layout when it moved, and sends the merged motion, once an animation frame. */
  private readonly poll = (): void => {
    this.frame = requestAnimationFrame(this.poll);
    this.rect = this.element.getBoundingClientRect();
    this.flush();

    const scale: number = window.devicePixelRatio;
    const layout: RenderViewportLayout = {
      backdrop: this.hole.getBackdrop(scale),
      rect: toNativeRect(this.rect, scale),
      scale,
    };
    const key: string = JSON.stringify(layout);

    if (key !== this.sent) {
      this.sent = key;
      this.viewport.setLayout(layout);
    }
  };

  private flush(): void {
    if (this.moved.dx !== 0 || this.moved.dy !== 0) {
      this.viewport.sendInput({ dx: this.moved.dx, dy: this.moved.dy, kind: EWorldInputEvent.DRAG_MOVE });
      this.moved = { dx: 0, dy: 0 };
    }

    if (this.hovered) {
      this.viewport.sendInput({ kind: EWorldInputEvent.HOVER, x: this.hovered.x, y: this.hovered.y });
      this.hovered = null;
    }
  }

  private send(event: WorldInputEvent): void {
    // Whatever moved before this event goes first, so a drag's end never arrives ahead of its last move.
    this.flush();
    this.viewport.sendInput(event);
  }

  /** Where a pointer is, in css pixels from the element's top left corner. */
  private toPoint(pointer: PointerEvent): IRenderViewPoint {
    return { x: pointer.clientX - this.rect.left, y: pointer.clientY - this.rect.top };
  }

  /** What a drag pressed at a point does; everything moves the view until the world names more under the pointer. */
  private getAffordance(_point: IRenderViewPoint): ENativeViewportAffordance {
    return ENativeViewportAffordance.VIEW;
  }

  private startDrag(press: IPress, pointer: PointerEvent): void {
    press.phase = EPressPhase.DRAGGING;
    this.send({ button: press.button, kind: EWorldInputEvent.DRAG_START, modifiers: press.modifiers });
    // The travel before it became a drag is its first move, so the view does not jump.
    this.track(press, pointer);

    this.resting ??= this.element.style.cursor;
    this.element.style.cursor = DRAG_CURSOR;

    const affordance: ENativeViewportAffordance = this.getAffordance({
      x: press.originX - this.rect.left,
      y: press.originY - this.rect.top,
    });

    switch (affordance) {
      case ENativeViewportAffordance.VIEW:
        // A turn reads the mouse's raw counts, as the game does; a pan keeps the system's motion, following the hand.
        return this.lock(press, this.isTurn(press));

      default:
        assertExhaustive(affordance);
    }
  }

  /** Whether a drag turns the camera rather than panning it, as the camera's controller tells them apart. */
  private isTurn(press: IPress): boolean {
    const kind: WorldCamera["kind"] = this.viewport.getCameraKind();

    switch (kind) {
      // Shift is the fly camera's boost, held while it looks around.
      case EWorldCamera.FLY:
        return press.button === 0;

      case EWorldCamera.ORBIT:
        return press.button === 0 && !press.modifiers.shiftKey && !press.modifiers.ctrlKey;

      default:
        assertExhaustive(kind);
    }
  }

  /** Gathers how far a dragging pointer moved: by its motion while locked, where its position stands still. */
  private track(press: IPress, pointer: PointerEvent): void {
    if (press.lock === EPressLock.LOCKED) {
      this.moved.dx += pointer.movementX;
      this.moved.dy += pointer.movementY;

      return;
    }

    this.moved.dx += pointer.clientX - press.lastX;
    this.moved.dy += pointer.clientY - press.lastY;
    press.lastX = pointer.clientX;
    press.lastY = pointer.clientY;
  }

  private lock(press: IPress, isRaw: boolean): void {
    press.lock = EPressLock.REQUESTED;

    void requestNativePointerLock(this.element, isRaw).then((isLocked: boolean) => {
      // A drag over before its lock arrived lets it go at once, unless another drag holds it by now.
      if (this.press !== press || press.phase !== EPressPhase.DRAGGING) {
        if (isLocked && this.press?.phase !== EPressPhase.DRAGGING) {
          this.unlock();
        }

        return;
      }

      // Refused, the drag goes on unlocked with the cursor showing.
      if (!isLocked) {
        press.lock = EPressLock.NONE;
      } else if (this.element.ownerDocument.pointerLockElement === this.element) {
        press.lock = EPressLock.LOCKED;
      }
    });
  }

  private unlock(): void {
    if (this.element.ownerDocument.pointerLockElement === this.element) {
      this.element.ownerDocument.exitPointerLock();
    }
  }

  private endDrag(press: IPress, reason: EWorldDragEndReason): void {
    if (press.phase !== EPressPhase.DRAGGING) {
      return;
    }

    press.lock = EPressLock.NONE;
    this.send({ kind: EWorldInputEvent.DRAG_END, reason });
    this.unlock();

    if (this.resting !== null) {
      this.element.style.cursor = this.resting;
      this.resting = null;
    }
  }

  /** Takes the press away while its button may still be held: nothing more is heard of it until it is let go. */
  private cancel(): void {
    if (this.press) {
      this.endDrag(this.press, EWorldDragEndReason.CANCELLED);
      this.press.phase = EPressPhase.CANCELLED;
    }
  }

  private click(pointer: PointerEvent): void {
    const point: IRenderViewPoint = this.toPoint(pointer);
    const isSecond: boolean =
      this.clicked !== null &&
      pointer.timeStamp - this.clicked.time <= RENDER_DOUBLE_CLICK_TIME &&
      Math.hypot(point.x - this.clicked.x, point.y - this.clicked.y) <= RENDER_CLICK_SLOP;

    this.clicked = isSecond ? null : { time: pointer.timeStamp, x: point.x, y: point.y };

    if (!isSecond) {
      this.onClick?.(point, pointer.altKey);
    }
  }

  /**
   * Hears the lock taken away mid-drag, by Escape or another window, which ends the drag rather than going on, and lets
   * go of every key: WebView2 drops the key-ups of keys released around the Escape that ends a lock.
   */
  private readonly onPointerLockChange = (): void => {
    const press: Nullable<IPress> = this.press;

    if (press?.phase !== EPressPhase.DRAGGING) {
      return;
    }

    if (this.element.ownerDocument.pointerLockElement === this.element) {
      press.lock = EPressLock.LOCKED;
    } else if (press.lock === EPressLock.LOCKED) {
      this.cancel();
      this.send({ kind: EWorldInputEvent.RELEASE_ALL });
    }
  };

  private readonly listeners: Readonly<Record<string, (event: Event) => void>> = {
    // A viewport that loses focus hears no key-ups, so nothing it heard held is held any more.
    blur: (): void => {
      this.cancel();
      this.send({ kind: EWorldInputEvent.RELEASE_ALL });
    },
    contextmenu: (event: Event): void => {
      event.preventDefault();
      this.send({ kind: EWorldInputEvent.CONTEXT_MENU });
    },
    keydown: (event: Event): void => {
      const key: KeyboardEvent = event as KeyboardEvent;

      // Alt is a modifier of the scene's own, a pick's among them, so it stays here rather than reaching the window as
      // the key that opens its menu.
      if (key.key === "Alt") {
        key.stopPropagation();
      }

      // A held key repeats; the renderer already holds it.
      if (!key.repeat) {
        this.send({ code: key.code, kind: EWorldInputEvent.KEY_DOWN });
      }
    },
    keyup: (event: Event): void => this.send({ code: (event as KeyboardEvent).code, kind: EWorldInputEvent.KEY_UP }),
    // A capture dropped mid-press takes the press away, unless a lock took it, which it does as it engages.
    lostpointercapture: (): void => {
      if (this.press && this.press.lock === EPressLock.NONE) {
        this.cancel();
      }
    },
    pointercancel: (event: Event): void => {
      if (this.press?.pointerId === (event as PointerEvent).pointerId) {
        this.cancel();
        this.press = null;
      }
    },
    pointerdown: (event: Event): void => {
      const pointer: PointerEvent = event as PointerEvent;

      // No text is selected or dragged from a press on the scene, which would take the drag that follows from it.
      event.preventDefault();
      clearWindowTextSelection();
      this.element.focus({ preventScroll: true });

      // One press at a time, of the first pointer down: a second finger would fight it.
      if (!pointer.isPrimary || (this.press && this.press.phase !== EPressPhase.CANCELLED)) {
        return;
      }

      this.element.setPointerCapture?.(pointer.pointerId);
      this.press = {
        button: pointer.button,
        lastX: pointer.clientX,
        lastY: pointer.clientY,
        lock: EPressLock.NONE,
        modifiers: {
          altKey: pointer.altKey,
          ctrlKey: pointer.ctrlKey,
          metaKey: pointer.metaKey,
          shiftKey: pointer.shiftKey,
        },
        originX: pointer.clientX,
        originY: pointer.clientY,
        phase: EPressPhase.PENDING,
        pointerId: pointer.pointerId,
      };
    },
    // The viewport under the pointer hears its keys without a click first; leaving it keeps the focus where it is.
    pointerenter: (event: Event): void => {
      if (!this.press && canFocusViewportOnHover(event as PointerEvent)) {
        this.element.focus({ preventScroll: true });
      }
    },
    pointermove: (event: Event): void => {
      const pointer: PointerEvent = event as PointerEvent;
      const press: Nullable<IPress> = this.press;

      if (!press) {
        this.hovered = this.toPoint(pointer);

        return;
      }

      if (pointer.pointerId !== press.pointerId) {
        return;
      }

      // A move with no button held ends a press whose release never arrived. Not while locked: a raw lock's moves
      // report no buttons in WebView2, and a locked pointer's release always reaches the lock.
      if (pointer.buttons === 0 && press.lock === EPressLock.NONE) {
        this.endDrag(press, EWorldDragEndReason.RELEASED);
        this.press = null;

        return;
      }

      switch (press.phase) {
        case EPressPhase.PENDING:
          if (Math.hypot(pointer.clientX - press.originX, pointer.clientY - press.originY) > RENDER_CLICK_SLOP) {
            this.startDrag(press, pointer);
          }

          return;

        case EPressPhase.DRAGGING:
          return this.track(press, pointer);

        case EPressPhase.CANCELLED:
          return;

        default:
          assertExhaustive(press.phase);
      }
    },
    pointerup: (event: Event): void => {
      const pointer: PointerEvent = event as PointerEvent;
      const press: Nullable<IPress> = this.press;

      if (press?.pointerId !== pointer.pointerId) {
        return;
      }

      this.press = null;

      // Only the main button is a click; it stayed within the slop, or it would be dragging.
      if (press.phase === EPressPhase.PENDING && press.button === 0 && pointer.button === 0) {
        this.click(pointer);
      }

      this.endDrag(press, EWorldDragEndReason.RELEASED);
    },
    wheel: (event: Event): void => {
      const wheel: WheelEvent = event as WheelEvent;

      event.preventDefault();
      this.send({
        deltaMode: wheel.deltaMode,
        deltaX: wheel.deltaX,
        deltaY: wheel.deltaY,
        kind: EWorldInputEvent.WHEEL,
      });
    },
  };
}
