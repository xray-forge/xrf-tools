import { Channel } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { assertExhaustive, Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import { CaptureOutcome, RenderPickRequest, ViewportEvent } from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import {
  ERenderViewportEvent,
  RenderAppliedReport,
  RenderCubeCapture,
  RenderFrameReport,
  RenderLevelHit,
  RenderLoadReport,
  RenderMapCapture,
  RenderOverlay,
  RenderSelection,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportId,
  RenderViewportLayout,
} from "@/core/ipc/types/xrf-renderer";
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
} from "@/core/ipc/types/xrf-world";
import { noop } from "@/lib/callbacks/noop";

/** A texture laid on a body, as the renderer is asked to draw one, or null for none. */
export type TNativeTextureRequest = Parameters<typeof renderCommands.showTexture>[1];

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

    this.attached = renderCommands.attachViewport(getCurrentWindow().label, events).then(
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
  public async pick(request: RenderPickRequest): Promise<Nullable<RenderLevelHit>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      return await renderCommands.pick(id, request);
    } catch (error: unknown) {
      this.noteFailure("pick", error);

      return null;
    }
  }

  /**
   * Counts what each shader table entry of the viewport's level draws across the sectors resident.
   *
   * @returns Every entry something draws, or none for a viewport not attached; rejects for a measure that failed.
   */
  public async measureSurfaces(): Promise<Array<WorldSurfaceGeometry>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return [];
    }

    try {
      return await renderCommands.measureSurfaces(id);
    } catch (error: unknown) {
      this.noteFailure("measureSurfaces", error);

      throw error;
    }
  }

  /**
   * @returns What the viewport's level could not draw the way it asked; nothing for a viewport not attached; rejects
   *   for a description that failed.
   */
  public async describeProblems(): Promise<WorldLevelProblems> {
    const id: Nullable<RenderViewportId> = await this.attached;
    const none: WorldLevelProblems = { models: [], sectors: [], skipped: [] };

    if (id === null || this.isDisposed) {
      return none;
    }

    try {
      return await renderCommands.describeProblems(id);
    } catch (error: unknown) {
      this.noteFailure("describeProblems", error);

      throw error;
    }
  }

  /**
   * @param object - A spawned object, by its index among the level's.
   * @returns Its bounding sphere in renderer space, centre then radius; null until its model is drawn.
   */
  public async locateSpawnObject(object: number): Promise<Nullable<[number, number, number, number]>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      const sphere = await renderCommands.locateSpawnObject(id, object);

      return sphere ? [sphere[0] ?? 0, sphere[1] ?? 0, sphere[2] ?? 0, sphere[3] ?? 0] : null;
    } catch (error: unknown) {
      this.noteFailure("locateSpawnObject", error);

      return null;
    }
  }

  /**
   * Says what became of every texture the viewport's level samples.
   *
   * @returns Each texture's reference and state, or none for a viewport not attached; rejects for a description that
   *   failed.
   */
  public async describeTextures(): Promise<Array<RenderTextureReport>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return [];
    }

    try {
      return await renderCommands.describeTextures(id);
    } catch (error: unknown) {
      this.noteFailure("describeTextures", error);

      throw error;
    }
  }

  /**
   * Writes the viewport's next finished frame to a PNG file: the whole viewport at its drawn size, without its
   * overlays or its selection's outline.
   *
   * @param path - The file to write.
   * @returns Whether it was written, which a viewport not attached is not; rejects for a screenshot that failed.
   */
  public async saveScreenshot(path: string): Promise<boolean> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return false;
    }

    try {
      await renderCommands.saveScreenshot(id, path);

      return true;
    } catch (error: unknown) {
      this.noteFailure("saveScreenshot", error);

      throw error;
    }
  }

  /**
   * Captures a level map in the viewport, tile by tile in place of its frames, and writes it to a PNG file.
   *
   * @param capture - What it is asked to capture.
   * @param path - The file to write.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null for a viewport not attached; rejects for a capture that failed.
   */
  public async captureMap(
    capture: RenderMapCapture,
    path: string,
    job: [string, Channel<JobProgress>]
  ): Promise<Nullable<CaptureOutcome>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      return await renderCommands.captureMap({ viewport: id, capture, path }, ...job);
    } catch (error: unknown) {
      this.noteFailure("captureMap", error);

      throw error;
    }
  }

  /**
   * Captures a cubemap around the viewport's camera, face by face in place of its frames, and writes its cross to a
   * PNG file.
   *
   * @param capture - What it is asked to capture.
   * @param path - The file to write.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null for a viewport not attached; rejects for a capture that failed.
   */
  public async captureCube(
    capture: RenderCubeCapture,
    path: string,
    job: [string, Channel<JobProgress>]
  ): Promise<Nullable<CaptureOutcome>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      return await renderCommands.captureCube({ viewport: id, capture, path }, ...job);
    } catch (error: unknown) {
      this.noteFailure("captureCube", error);

      throw error;
    }
  }

  /**
   * Draws an open level, or none.
   *
   * @param sessionId - The level's open session, as the backend holds it, or null for no level.
   * @returns The scene drawn, which the viewport's load reports name; null for no level or a viewport not attached;
   *   rejects for a request that failed.
   */
  public showLevel(sessionId: Nullable<string>): Promise<Nullable<number>> {
    return this.show("showLevel", (id: RenderViewportId) => renderCommands.showLevel(id, sessionId));
  }

  /**
   * Draws an open model alone, at the origin, or none.
   *
   * @param sessionId - The model's open session, as the backend holds it, or null for no model.
   * @param detail - How far down each submesh's collapse chain to draw: zero its finest level, one its coarsest.
   * @returns The scene drawn, which the viewport's load reports name; null for no model or a viewport not attached;
   *   rejects for a request that failed.
   */
  public showModel(sessionId: Nullable<string>, detail: number): Promise<Nullable<number>> {
    return this.show("showModel", (id: RenderViewportId) => renderCommands.showModel(id, sessionId, detail));
  }

  /**
   * Draws a texture laid on a body, or none.
   *
   * @param request - The texture, the body and how it is laid on it, or null for none.
   * @returns The scene drawn, which the viewport's load reports name; null for no texture or a viewport not attached;
   *   rejects for a request that failed.
   */
  public showTexture(request: TNativeTextureRequest): Promise<Nullable<number>> {
    return this.show("showTexture", (id: RenderViewportId) => renderCommands.showTexture(id, request));
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
   * @param command - The show sent, which a failure is told under.
   * @param send - Sends it to the attached viewport.
   * @returns The scene it drew, or null for none or a viewport not attached; rejects for a show that failed.
   */
  private async show(
    command: string,
    send: (id: RenderViewportId) => Promise<Nullable<number>>
  ): Promise<Nullable<number>> {
    const id: Nullable<RenderViewportId> = await this.attached;

    if (id === null || this.isDisposed) {
      return null;
    }

    try {
      return (await send(id)) ?? null;
    } catch (error: unknown) {
      this.noteFailure(command, error);

      throw error;
    }
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
