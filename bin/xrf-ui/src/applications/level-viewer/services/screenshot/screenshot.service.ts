import { save } from "@tauri-apps/plugin-dialog";
import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND } from "@/applications/level-viewer/commands";
import { toLevelViewerFileStem } from "@/applications/level-viewer/lib/files/level-viewer-file-stem";
import {
  copyLevelViewerImage,
  ILevelViewerImage,
  readLevelViewerImage,
} from "@/applications/level-viewer/lib/files/level-viewer-image";
import {
  ILevelViewerScreenshot,
  ILevelViewerScreenshotReport,
  toLevelViewerScreenshotReportPath,
} from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import {
  ILevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { KeybindCommand } from "@/core/commands";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { systemCommands } from "@/core/ipc/commands/system";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { Logger } from "@/lib/logging";

/** Timed reports waited for once the GPU timing is switched on: the first may still hold frames from before it. */
const TIMED_REPORTS: number = 2;

/** Milliseconds a screenshot waits for its timed reports before it reports the frame as it stands. */
const TIMED_REPORTS_TIMEOUT: number = 3000;

/**
 * The level viewer's screenshots: the viewport's finished frame saved at once to the screenshots folder, one at a time,
 * with a report beside it of the view it shows and what its frame cost, its passes timed on the GPU for it.
 */
@Injectable()
export class LevelViewerScreenshotService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Whether a screenshot is being taken, during which another is not. */
  @RefObservable()
  public isTaking: boolean = false;

  /** The last screenshot taken, until it is deleted. */
  @RefObservable()
  public shot: Nullable<ILevelViewerScreenshot> = null;

  /** Why the last screenshot failed, until it is dismissed or another is taken. */
  @RefObservable()
  public failure: Nullable<string> = null;

  /** Whether the last screenshot is shown whole, in its dialog. */
  @RefObservable()
  public isShown: boolean = false;

  public constructor(
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly snapshotService: LevelViewerSnapshotService = inject(LevelViewerSnapshotService),
    private readonly renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {}

  @OnDeactivation()
  public onDeactivation(): void {
    this.replace(null);
  }

  /** Whether a screenshot can be taken now: a level is open and none is being taken. */
  @Computed()
  public get canTake(): boolean {
    return !this.isTaking && this.snapshotService.level !== null;
  }

  /**
   * @param isShown - Whether to show the last screenshot whole.
   */
  @BoundAction()
  public setShown(isShown: boolean): void {
    this.isShown = isShown && this.shot !== null;
  }

  @BoundAction()
  public dismissFailure(): void {
    this.failure = null;
  }

  /**
   * Saves the viewport's next finished frame to the screenshots folder, then its report once its frame's cost is read.
   *
   * @returns The screenshot, or null where none was taken: one is being taken, no level is open, or it failed.
   */
  @KeybindCommand(TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerScreenshotService) => service.canTake,
  })
  public async take(): Promise<Nullable<ILevelViewerScreenshot>> {
    const level: Nullable<string> = this.snapshotService.level;

    if (!this.canTake || !level) {
      return null;
    }

    const isTimed: boolean = this.renderSettingsService.isGpuTimed;

    runInAction(() => {
      this.isTaking = true;
      this.failure = null;
    });

    // Timed from now, so its reports come in while the screenshot is read back.
    if (!isTimed) {
      this.renderSettingsService.setGpuTimed(true);
    }

    const timedFrame: Promise<RenderFrameReport> = isTimed
      ? Promise.resolve(this.renderService.frame)
      : this.waitForTimedFrame();

    try {
      const path: string = await levelsCommands.findScreenshotPath(toLevelViewerFileStem(level, new Date()));

      if (!(await this.renderService.saveScreenshot(path))) {
        throw new Error("The level viewer has no viewport attached");
      }

      // The view as it stood for the frame, before the camera moves on.
      const view: Promise<ILevelViewerSnapshot> = this.snapshotService.exportSnapshot();
      const shot: ILevelViewerScreenshot = await this.read(path);

      this.replace(shot);
      this.log.info("Took a screenshot:", path);
      await this.writeReport(shot, await view, await timedFrame, !isTimed);

      return this.shot;
    } catch (error: unknown) {
      this.fail("take a screenshot", error);

      return null;
    } finally {
      if (!isTimed) {
        this.renderSettingsService.setGpuTimed(false);
      }

      runInAction(() => {
        this.isTaking = false;
      });
    }
  }

  /** Deletes the last screenshot and its report for good. */
  public remove(): Promise<void> {
    return this.attempt("delete the screenshot", async (shot: ILevelViewerScreenshot) => {
      await levelsCommands.deleteScreenshot(shot.path);
      this.log.info("Deleted a screenshot:", shot.path);
      this.replace(null);
    });
  }

  /** Shows the last screenshot selected in its folder. */
  public reveal(): Promise<void> {
    return this.attempt("show the screenshot", (shot: ILevelViewerScreenshot) => systemCommands.revealPath(shot.path));
  }

  /** Shows the last screenshot's report selected in its folder. */
  public revealReport(): Promise<void> {
    return this.attempt("show the screenshot's report", (shot: ILevelViewerScreenshot) =>
      systemCommands.revealPath(shot.reportPath)
    );
  }

  /** Puts the last screenshot's path on the clipboard. */
  public copyPath(): Promise<void> {
    return this.attempt("copy the screenshot's path", (shot: ILevelViewerScreenshot) =>
      navigator.clipboard.writeText(shot.path)
    );
  }

  /** Puts the last screenshot's image on the clipboard. */
  public copyImage(): Promise<void> {
    return this.attempt("copy the screenshot", copyLevelViewerImage);
  }

  /** Copies the last screenshot where a save dialog answers. */
  public saveAs(): Promise<void> {
    return this.attempt("save the screenshot", async (shot: ILevelViewerScreenshot) => {
      const path: Nullable<string> = await save({
        defaultPath: shot.name,
        filters: [{ extensions: [EXrayExtension.PNG], name: "PNG image" }],
        title: "Save screenshot as",
      });

      if (path) {
        await levelsCommands.copyScreenshot(shot.path, path);
        this.log.info("Saved a screenshot as:", path);
      }
    });
  }

  /** Does `action` with the last screenshot, if any; a failure is logged and kept as `failure`, as `what` says. */
  private async attempt(what: string, action: (shot: ILevelViewerScreenshot) => Promise<unknown>): Promise<void> {
    if (this.shot) {
      await action(this.shot).catch((error: unknown) => this.fail(what, error));
    }
  }

  @BoundAction()
  private fail(what: string, error: unknown): void {
    this.log.error(`Failed to ${what}:`, error);
    this.failure = `Failed to ${what}: ${error instanceof Error ? error.message : String(error)}`;
  }

  /** The screenshot written to `path`, read back for the page. */
  private async read(path: string): Promise<ILevelViewerScreenshot> {
    const image: ILevelViewerImage = await readLevelViewerImage(path);

    return { ...image, frame: null, reportPath: toLevelViewerScreenshotReportPath(path) };
  }

  /**
   * Writes a screenshot's report beside it and keeps its frame's cost with it.
   *
   * @param shot - The screenshot.
   * @param view - The view it shows.
   * @param frame - What its frame cost.
   * @param isTimingSwitched - Whether the GPU timing was switched on for it alone.
   */
  private async writeReport(
    shot: ILevelViewerScreenshot,
    view: ILevelViewerSnapshot,
    frame: RenderFrameReport,
    isTimingSwitched: boolean
  ): Promise<void> {
    const report: ILevelViewerScreenshotReport = {
      ...view,
      screenshot: { file: shot.name, height: shot.height, isTimingSwitched, width: shot.width },
      frame,
    };

    await levelsCommands.saveView(shot.reportPath, serializeLevelViewerSnapshot(report));
    runInAction(() => {
      if (this.shot === shot) {
        this.shot = { ...shot, frame };
      }
    });
  }

  /** Holds `shot` as the last screenshot, letting the one it replaces go. */
  @BoundAction()
  private replace(shot: Nullable<ILevelViewerScreenshot>): void {
    if (this.shot) {
      URL.revokeObjectURL(this.shot.url);
    }

    this.shot = shot;
    this.isShown = this.isShown && shot !== null;
  }

  /** The viewport's frame report once its passes are timed, or the latest one should no timed report come in time. */
  private waitForTimedFrame(): Promise<RenderFrameReport> {
    return new Promise((resolve: (frame: RenderFrameReport) => void) => {
      let timed: number = 0;

      const finish = (): void => {
        clearTimeout(timeout);
        dispose();
        resolve(this.renderService.frame);
      };
      const timeout: ReturnType<typeof setTimeout> = setTimeout(finish, TIMED_REPORTS_TIMEOUT);
      const dispose: () => void = reaction(
        () => this.renderService.frame,
        (frame: RenderFrameReport) => {
          timed = frame.isGpuTimed ? timed + 1 : 0;

          if (timed >= TIMED_REPORTS) {
            finish();
          }
        }
      );
    });
  }
}
