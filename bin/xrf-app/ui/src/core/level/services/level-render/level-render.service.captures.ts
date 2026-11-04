import { Channel } from "@tauri-apps/api/core";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { CaptureOutcome } from "@xrf/ipc/types/xrf-app-render";
import { RenderCubeCapture, RenderMapCapture, RenderViewportId } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { ILevelRenderJob } from "@/core/level/lib/render/level-render-job";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelViewService } from "@/core/level/services/level-view";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/**
 * What the level's viewport writes to files: its next frame, a map of the level, a cubemap about the camera, a
 * recording of it with its sound.
 */
export class LevelRenderCaptures {
  /**
   * @param context - The viewport the captures are taken of.
   * @param viewService - Whether helpers are drawn and asked into screenshots.
   */
  public constructor(
    private readonly context: ILevelRenderContext,
    private readonly viewService: LevelViewService
  ) {}

  /**
   * @param path - The PNG file to write the viewport's next finished frame to, without its overlays, its helpers over
   *   it where they are drawn and asked into screenshots.
   * @returns Whether it was written, which it is not with no viewport attached; rejects for a screenshot that failed.
   */
  public async saveScreenshot(path: string): Promise<boolean> {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();
    const { isShown, isScreenshotHelped } = this.viewService.helpers;

    return viewport
      ? viewport.ask(
          "saveScreenshot",
          async (id: RenderViewportId) => {
            await levelsCommands.saveScreenshot({
              path,
              screenshot: { isHelped: isShown && isScreenshotHelped },
              viewport: id,
            });

            return true;
          },
          false
        )
      : false;
  }

  /**
   * @param capture - The level map asked for.
   * @param path - The PNG file to write it to.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null with no viewport attached; rejects for a capture that failed.
   */
  public async captureMap(
    capture: RenderMapCapture,
    path: string,
    job: ILevelRenderJob
  ): Promise<Nullable<CaptureOutcome>> {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    return viewport
      ? viewport.ask<Nullable<CaptureOutcome>>(
          "captureMap",
          (id: RenderViewportId) => levelsCommands.captureMap({ viewport: id, capture, path }, job.id, job.progress),
          null
        )
      : null;
  }

  /**
   * @param name - What the recording's file in the recordings folder is named for, without its extension.
   * @param progress - Where the recording tells where it stands, and why where it ends by itself.
   * @returns The file it records into, or null with no viewport attached; rejects for a recording that cannot start.
   */
  public async startRecording(name: string, progress: Channel<RecordingProgress>): Promise<Nullable<string>> {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    return viewport
      ? viewport.ask<Nullable<string>>(
          "startRecording",
          (id: RenderViewportId) => levelsCommands.startRecording(id, name, progress),
          null
        )
      : null;
  }

  /**
   * @param capture - The cubemap asked for, around the camera.
   * @param path - The PNG file to write its cross to.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null with no viewport attached; rejects for a capture that failed.
   */
  public async captureCube(
    capture: RenderCubeCapture,
    path: string,
    job: ILevelRenderJob
  ): Promise<Nullable<CaptureOutcome>> {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    return viewport
      ? viewport.ask<Nullable<CaptureOutcome>>(
          "captureCube",
          (id: RenderViewportId) => levelsCommands.captureCube({ viewport: id, capture, path }, job.id, job.progress),
          null
        )
      : null;
  }
}
