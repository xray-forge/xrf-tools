import { Channel } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { assertExhaustive, Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import {
  ERenderViewportEvent,
  RenderCamera,
  RenderCameraCommand,
  RenderCameraPose,
  RenderFrameReport,
  RenderInputEvent,
  RenderViewportEvent,
  RenderViewportId,
  RenderViewportLayout,
} from "@/core/ipc/types/xrf-renderer";

/**
 * What a native viewport tells its page.
 */
export interface INativeViewportListener {
  /** What its recent frames cost. */
  onFrame(report: RenderFrameReport): void;
  /** Where its camera stands, while it moves and once after. */
  onCamera(pose: RenderCameraPose): void;
  /** Why it cannot be drawn. */
  onFailed(message: string): void;
}

/**
 * One viewport the renderer draws into this window, from its attach to its detach.
 * Every call waits for the attach and goes in the order it was made, so a camera set before the renderer answered is
 * still the camera it draws.
 */
export class NativeViewport {
  private readonly attached: Promise<Nullable<RenderViewportId>>;
  private isDisposed: boolean = false;

  public constructor(listener: INativeViewportListener) {
    const events: Channel<RenderViewportEvent> = new Channel<RenderViewportEvent>();

    events.onmessage = (event: RenderViewportEvent): void => {
      if (this.isDisposed) {
        return;
      }

      switch (event.kind) {
        case ERenderViewportEvent.FRAME:
          return listener.onFrame(event.report);

        case ERenderViewportEvent.CAMERA:
          return listener.onCamera(event.pose);

        case ERenderViewportEvent.FAILURE:
          return listener.onFailed(event.message);

        default:
          assertExhaustive(event);
      }
    };

    this.attached = renderCommands.attachViewport(getCurrentWindow().label, events).then(
      (id: RenderViewportId) => {
        // Released before the renderer answered: what it attached goes at once.
        if (this.isDisposed) {
          void renderCommands.detachViewport(id);

          return null;
        }

        return id;
      },
      (error: unknown) => {
        listener.onFailed(error instanceof Error ? error.message : String(error));

        return null;
      }
    );
  }

  public setLayout(layout: RenderViewportLayout): void {
    this.call((id: RenderViewportId) => renderCommands.setViewportLayout(id, layout));
  }

  public sendInput(event: RenderInputEvent): void {
    this.call((id: RenderViewportId) => renderCommands.sendInput(id, event));
  }

  public setCamera(camera: RenderCamera): void {
    this.call((id: RenderViewportId) => renderCommands.setCamera(id, camera));
  }

  public commandCamera(command: RenderCameraCommand): void {
    this.call((id: RenderViewportId) => renderCommands.commandCamera(id, command));
  }

  /** Stops drawing; the renderer lets its GPU go a few seconds after the last viewport. */
  public dispose(): void {
    if (this.isDisposed) {
      return;
    }

    this.isDisposed = true;
    void this.attached.then((id: Nullable<RenderViewportId>) => {
      if (id !== null) {
        return renderCommands.detachViewport(id);
      }
    });
  }

  private call(send: (id: RenderViewportId) => Promise<void>): void {
    void this.attached.then((id: Nullable<RenderViewportId>) => {
      if (id !== null && !this.isDisposed) {
        return send(id).catch(() => {
          // A command to a viewport the renderer dropped changes nothing: its failure is reported on its own.
        });
      }
    });
  }
}
