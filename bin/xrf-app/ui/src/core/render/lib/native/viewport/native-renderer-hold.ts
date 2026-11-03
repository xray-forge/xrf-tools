import { renderCommands } from "@xrf/ipc/commands/render";
import { RenderHoldId } from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

/**
 * A hold on the native renderer, keeping its GPU started between viewports: a tool whose viewports come and go takes
 * it as it opens, so the first viewport and every one after it draws without waiting for the GPU to start.
 */
export class NativeRendererHold {
  /** The hold asked for, answered with its number, or null for one the renderer refused. */
  private held: Nullable<Promise<Nullable<RenderHoldId>>> = null;

  /**
   * @param log - The owner's logger, which a refused hold is told on.
   */
  public constructor(private readonly log: Logger) {}

  /** Holds the renderer, unless held already. */
  public take(): void {
    if (this.held) {
      return;
    }

    this.held = renderCommands.hold().catch((error: unknown) => {
      // Nothing breaks without it: the GPU starts with the first viewport, as it does unheld.
      this.log.warn("The native renderer cannot be held:", error);

      return null;
    });
  }

  /** Lets the hold go, once the renderer answered it where it has not yet. */
  public release(): void {
    const held: Nullable<Promise<Nullable<RenderHoldId>>> = this.held;

    this.held = null;

    void held?.then((hold: Nullable<RenderHoldId>) => {
      if (hold !== null) {
        return renderCommands.releaseHold(hold).catch((error: unknown) => {
          this.log.warn("The native renderer's hold cannot be released:", error);
        });
      }
    });
  }
}
