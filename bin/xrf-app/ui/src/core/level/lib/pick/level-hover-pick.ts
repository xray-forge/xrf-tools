import { call, TFlow } from "@xrf/flow";
import { RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/**
 * The pointer's picks over a viewport, one in flight at a time: a point arriving meanwhile waits for it and is picked
 * next, and a newer one, its flow taking over, drops the point waiting, so the pointer is answered as fast as a pick
 * comes back and never queues the renderer's readbacks.
 */
export class LevelHoverPick {
  /** The hover pick in flight, settled either way, which the next hover waits for; null while none is. */
  private hovering: Nullable<Promise<void>> = null;

  /**
   * Picks what is drawn under a point once the pick in flight has settled, never passing glass.
   *
   * @param point - Where, in css pixels from the viewport's top left corner.
   * @param getViewport - The viewport to ask once the pick in flight has settled, or null where it, or what it shows,
   *   went meanwhile.
   * @returns What is drawn there; null for nothing, or for no viewport to ask.
   */
  public *pick(point: IRenderViewPoint, getViewport: () => Nullable<NativeViewport>): TFlow<Nullable<RenderLevelHit>> {
    if (this.hovering) {
      yield* call(this.hovering);
    }

    // The pick in flight may have outlived the viewport, or what it showed.
    const viewport: Nullable<NativeViewport> = getViewport();

    if (!viewport) {
      return null;
    }

    const picking: Promise<Nullable<RenderLevelHit>> = viewport.pick({ x: point.x, y: point.y, isThrough: false });

    const settled: Promise<void> = picking
      .then(
        () => undefined,
        () => undefined
      )
      .then(() => {
        if (this.hovering === settled) {
          this.hovering = null;
        }
      });

    this.hovering = settled;

    return yield* call(picking);
  }

  /** Forgets the pick in flight, which named what was shown. */
  public reset(): void {
    this.hovering = null;
  }
}
