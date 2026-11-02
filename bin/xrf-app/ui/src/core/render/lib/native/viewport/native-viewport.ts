import { Channel } from "@tauri-apps/api/core";
import { renderCommands } from "@xrf/ipc/commands/render";
import { RenderPickRequest, ViewportEvent } from "@xrf/ipc/types/xrf-app-render";
import {
  ERenderViewportEvent,
  RenderAppliedReport,
  RenderFrameReport,
  RenderLevelHit,
  RenderLoadReport,
  RenderOverlay,
  RenderSelection,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportId,
  RenderViewportLayout,
} from "@xrf/ipc/types/xrf-renderer";
import {
  EWorldCamera,
  EWorldViewportEvent,
  EWorldWeatherTransition,
  WorldCamera,
  WorldCameraCommand,
  WorldCameraPose,
  WorldInputEvent,
  WorldLevelProblems,
  WorldModelPose,
  WorldSurfaceGeometry,
  WorldToggles,
  WorldWeatherControl,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@xrf/ipc/types/xrf-world";
import { assertExhaustive, Nullable } from "@xrf/types";

import { noop } from "@/lib/callbacks/noop";

/**
 * What a native viewport tells its page.
 */
export interface INativeViewportListener {
  /** What its recent frames cost. */
  onFrame(report: RenderFrameReport): void;
  /** What its frames are drawn with, as the renderer resolved what it was asked, as that changes. */
  onApplied(report: RenderAppliedReport): void;
  /** Where its camera stands, while it moves and once after. */
  onCamera(pose: WorldCameraPose): void;
  /** How far its scene has loaded, as that changes. */
  onLoad(report: RenderLoadReport): void;
  /** Where its weather stands, as that changes; null while none plays. */
  onWeather(report: Nullable<WorldWeatherReport>): void;
  /** Why it cannot be drawn. */
  onFailed(message: string): void;
  /** A command it was sent failed: said once per command, so one sent every frame says so once. */
  onCommandFailed(command: string, error: unknown): void;
}

/**
 * One viewport the renderer draws into this window, from its attach to its detach.
 * Every call waits for the attach and goes in the order it was made, so a camera set before the renderer answered is
 * still the camera it draws.
 */
export class NativeViewport {
  private readonly attached: Promise<Nullable<RenderViewportId>>;
  private isDisposed: boolean = false;
  /** The kind of camera last described; the renderer flies one until it is told. */
  private cameraKind: WorldCamera["kind"] = EWorldCamera.FLY;
  /** The commands whose failure was told. */
  private readonly failed: Set<string> = new Set();
  private readonly listener: INativeViewportListener;

  public constructor(listener: INativeViewportListener) {
    this.listener = listener;

    const events: Channel<ViewportEvent> = new Channel<ViewportEvent>();

    events.onmessage = (event: ViewportEvent): void => {
      if (this.isDisposed) {
        return;
      }

      switch (event.kind) {
        case ERenderViewportEvent.FRAME:
          return listener.onFrame(event.report);

        case ERenderViewportEvent.APPLIED:
          return listener.onApplied(event.report);

        case EWorldViewportEvent.CAMERA:
          return listener.onCamera(event.pose);

        case ERenderViewportEvent.LOAD:
          return listener.onLoad(event.report);

        case EWorldViewportEvent.WEATHER:
          return listener.onWeather(event.report);

        case ERenderViewportEvent.FAILURE:
          return listener.onFailed(event.message);

        default:
          assertExhaustive(event);
      }
    };

    this.attached = renderCommands.attachViewport(events).then(
      (id: RenderViewportId) => {
        // Released before the renderer answered: what it attached goes at once.
        if (this.isDisposed) {
          void renderCommands.detachViewport(id).catch(noop);

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
    this.call("setViewportLayout", (id: RenderViewportId) => renderCommands.setViewportLayout(id, layout));
  }

  public sendInput(event: WorldInputEvent): void {
    this.call("sendInput", (id: RenderViewportId) => renderCommands.sendInput(id, event));
  }

  public setCamera(camera: WorldCamera): void {
    this.cameraKind = camera.kind;
    this.call("setCamera", (id: RenderViewportId) => renderCommands.setCamera(id, camera));
  }

  public getCameraKind(): WorldCamera["kind"] {
    return this.cameraKind;
  }

  public commandCamera(command: WorldCameraCommand): void {
    this.call("commandCamera", (id: RenderViewportId) => renderCommands.commandCamera(id, command));
  }

  public setViewOptions(options: RenderViewOptions): void {
    this.call("setViewOptions", (id: RenderViewportId) => renderCommands.setViewOptions(id, options));
  }

  /**
   * @param toggles - What of the shown level's world plays: its weather's rain, bolts and wind, its campfires, its
   *   ambient effects and which spawned groups stream in.
   */
  public setWorldToggles(toggles: WorldToggles): void {
    this.call("setWorldToggles", (id: RenderViewportId) => renderCommands.setWorldToggles(id, toggles));
  }

  /**
   * @param overlays - The helpers drawn over its frame from now on, in place of those before.
   */
  public setOverlays(overlays: Array<RenderOverlay>): void {
    this.call("setOverlays", (id: RenderViewportId) => renderCommands.setOverlays(id, overlays));
  }

  public setSelection(selection: Nullable<RenderSelection>): void {
    this.call("setSelection", (id: RenderViewportId) => renderCommands.setSelection(id, selection));
  }

  public playWeather(play: WorldWeatherPlay, transition: EWorldWeatherTransition): void {
    this.call("playWeather", (id: RenderViewportId) => renderCommands.playWeather(id, play, transition));
  }

  public setWeatherControl(control: WorldWeatherControl): void {
    this.call("setWeatherControl", (id: RenderViewportId) => renderCommands.setWeatherControl(id, control));
  }

  /**
   * @param time - Seconds since midnight to play the weather on from.
   */
  public seekWeather(time: number): void {
    this.call("seekWeather", (id: RenderViewportId) => renderCommands.seekWeather(id, time));
  }

  /**
   * @param name - The effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  public playWeatherEffect(name: Nullable<string>): void {
    this.call("playWeatherEffect", (id: RenderViewportId) => renderCommands.playWeatherEffect(id, name));
  }

  /** Plays a weather ambient effect near the camera at once, ending the one playing; none plays indoors. */
  public playAmbientEffect(): void {
    this.call("playAmbientEffect", (id: RenderViewportId) => renderCommands.playAmbientEffect(id));
  }

  /**
   * Names what the viewport's level draws under a point.
   *
   * @param request - The point, in css pixels from the viewport's top left corner, and whether glass is passed by.
   * @returns What was hit, or null for nothing, a viewport not attached or a pick that failed.
   */
  public pick(request: RenderPickRequest): Promise<Nullable<RenderLevelHit>> {
    return this.ask("pick", (id: RenderViewportId) => renderCommands.pick(id, request), null).catch(() => null);
  }

  /**
   * Counts what each shader table entry of the viewport's level draws across the sectors resident.
   *
   * @returns Every entry something draws, or none for a viewport not attached; rejects for a measure that failed.
   */
  public measureSurfaces(): Promise<Array<WorldSurfaceGeometry>> {
    return this.ask("measureSurfaces", (id: RenderViewportId) => renderCommands.measureSurfaces(id), []);
  }

  /**
   * @returns What the viewport's level could not draw the way it asked; nothing for a viewport not attached; rejects
   *   for a description that failed.
   */
  public describeProblems(): Promise<WorldLevelProblems> {
    return this.ask("describeProblems", (id: RenderViewportId) => renderCommands.describeProblems(id), {
      models: [],
      sectors: [],
      skipped: [],
    });
  }

  /**
   * @param object - A spawned object, by its index among the level's.
   * @returns Its bounding sphere in renderer space, centre then radius; null until its model is drawn, for a viewport
   *   not attached or a locate that failed.
   */
  public async locateSpawnObject(object: number): Promise<Nullable<[number, number, number, number]>> {
    const sphere = await this.ask(
      "locateSpawnObject",
      (id: RenderViewportId) => renderCommands.locateSpawnObject(id, object),
      null
    ).catch(() => null);

    return sphere ? [sphere[0] ?? 0, sphere[1] ?? 0, sphere[2] ?? 0, sphere[3] ?? 0] : null;
  }

  /**
   * Says what became of every texture the viewport's level samples.
   *
   * @returns Each texture's reference and state, or none for a viewport not attached; rejects for a description that
   *   failed.
   */
  public describeTextureLoads(): Promise<Array<RenderTextureReport>> {
    return this.ask("describeTextureLoads", (id: RenderViewportId) => renderCommands.describeTextureLoads(id), []);
  }

  /**
   * Sends a domain's command about this viewport, such as the show of its document or a capture of what it draws, once
   * it is attached: the renderer knows no domain, so each domain names the viewport in a command of its own.
   *
   * @param command - What is sent, which a failure is told under.
   * @param send - Sends it to the attached viewport.
   * @param detached - What a viewport not attached, or released, answers instead.
   * @returns What it answered; rejects for a command that failed.
   */
  public async ask<T>(command: string, send: (id: RenderViewportId) => Promise<T>, detached: T): Promise<T> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return detached;
    }

    try {
      return await send(id);
    } catch (error: unknown) {
      this.noteFailure(command, error);

      throw error;
    }
  }

  /**
   * @param pose - How the viewport's skinned models stand from now on.
   */
  public poseModel(pose: WorldModelPose): void {
    this.call("poseModel", (id: RenderViewportId) => renderCommands.poseModel(id, pose));
  }

  /** Stops drawing; the renderer lets its GPU go a few seconds after the last viewport. */
  public dispose(): void {
    if (this.isDisposed) {
      return;
    }

    this.isDisposed = true;
    void this.attached.then((id: Nullable<RenderViewportId>) => {
      if (id !== null) {
        return renderCommands.detachViewport(id).catch(noop);
      }
    });
  }

  /**
   * @param command - What is sent, which a failure is told under.
   * @param send - Sends it to the attached viewport.
   */
  private call(command: string, send: (id: RenderViewportId) => Promise<void>): void {
    void this.attached.then((id: Nullable<RenderViewportId>) => {
      if (id !== null && !this.isDisposed) {
        return send(id).catch((error: unknown) => this.noteFailure(command, error));
      }
    });
  }

  /**
   * Tells a command's first failure; none once released, when the renderer dropping the viewport is expected.
   */
  private noteFailure(command: string, error: unknown): void {
    if (this.isDisposed || this.failed.has(command)) {
      return;
    }

    this.failed.add(command);
    this.listener.onCommandFailed(command, error);
  }
}
