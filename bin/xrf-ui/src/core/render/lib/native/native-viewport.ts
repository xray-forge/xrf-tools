import { Channel } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { assertExhaustive, Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import {
  ERenderViewportEvent,
  ERenderWeatherTransition,
  RenderCamera,
  RenderCameraCommand,
  RenderCameraPose,
  RenderFrameReport,
  RenderInputEvent,
  RenderLevelHit,
  RenderLoadReport,
  RenderSurfaceGeometry,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportEvent,
  RenderViewportId,
  RenderViewportLayout,
  RenderWeatherControl,
  RenderWeatherPlay,
  RenderWeatherReport,
} from "@/core/ipc/types/xrf-renderer";

/**
 * What a native viewport tells its page.
 */
export interface INativeViewportListener {
  /** What its recent frames cost. */
  onFrame(report: RenderFrameReport): void;
  /** Where its camera stands, while it moves and once after. */
  onCamera(pose: RenderCameraPose): void;
  /** How far its scene has loaded, as that changes. */
  onLoad(report: RenderLoadReport): void;
  /** Where its weather stands, as that changes; null while none plays. */
  onWeather(report: Nullable<RenderWeatherReport>): void;
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

        case ERenderViewportEvent.LOAD:
          return listener.onLoad(event.report);

        case ERenderViewportEvent.WEATHER:
          return listener.onWeather(event.report);

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

  public setViewOptions(options: RenderViewOptions): void {
    this.call((id: RenderViewportId) => renderCommands.setViewOptions(id, options));
  }

  public playWeather(play: RenderWeatherPlay, transition: ERenderWeatherTransition): void {
    this.call((id: RenderViewportId) => renderCommands.playWeather(id, play, transition));
  }

  public setWeatherControl(control: RenderWeatherControl): void {
    this.call((id: RenderViewportId) => renderCommands.setWeatherControl(id, control));
  }

  /**
   * @param time - Seconds since midnight to play the weather on from.
   */
  public seekWeather(time: number): void {
    this.call((id: RenderViewportId) => renderCommands.seekWeather(id, time));
  }

  /**
   * @param name - The effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  public playWeatherEffect(name: Nullable<string>): void {
    this.call((id: RenderViewportId) => renderCommands.playWeatherEffect(id, name));
  }

  /**
   * Names what the viewport's level draws under a point.
   *
   * @param x - Css pixels from the viewport's left edge.
   * @param y - Css pixels from its top edge.
   * @returns What was hit, or null for nothing, a viewport not attached or a pick that failed.
   */
  public async pick(x: number, y: number): Promise<Nullable<RenderLevelHit>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      return await renderCommands.pick(id, x, y);
    } catch {
      return null;
    }
  }

  /**
   * Counts what each shader table entry of the viewport's level draws across the sectors resident.
   *
   * @returns Every entry something draws, or none for a viewport not attached or a measure that failed.
   */
  public async measureSurfaces(): Promise<Array<RenderSurfaceGeometry>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return [];
    }

    try {
      return await renderCommands.measureSurfaces(id);
    } catch {
      return [];
    }
  }

  /**
   * Says what became of every texture the viewport's level samples.
   *
   * @returns Each texture's reference and state, or none for a viewport not attached or a description that failed.
   */
  public async describeTextures(): Promise<Array<RenderTextureReport>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return [];
    }

    try {
      return await renderCommands.describeTextures(id);
    } catch {
      return [];
    }
  }

  /**
   * Draws an open level, or none.
   *
   * @param sessionId - The level's open session, as the backend holds it, or null for no level.
   */
  public showLevel(sessionId: Nullable<string>): void {
    this.call((id: RenderViewportId) => renderCommands.showLevel(id, sessionId).then(() => undefined));
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
