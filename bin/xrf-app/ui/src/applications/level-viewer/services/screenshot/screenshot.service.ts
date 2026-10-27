import { save } from "@tauri-apps/plugin-dialog";
import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, reaction, RefObservable } from "@wirestate/mobx";
import { all, call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";
import { RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";
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
} from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";
import {
  ILevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { KeybindCommand } from "@/core/commands";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { noop } from "@/lib/callbacks/noop";

/** Timed reports waited for once the GPU timing is switched on: the first may still hold frames from before it. */
const TIMED_REPORTS: number = 2;

/** Milliseconds a screenshot waits for its timed reports before it reports the frame as it stands. */
const TIMED_REPORTS_TIMEOUT: number = 3000;

/** A wait for a timed frame report, and what ends it at once. */
interface ITimedFrameWait {
  frame: Promise<RenderFrameReport>;
  stop: () => void;
}

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
   * Saves the viewport's next finished frame to the screenshots folder, then its report once its frame's cost is read;
   * the passes are timed on the GPU meanwhile, the timing setting left as it is.
   *
   * One left running when the tool is left is cancelled, so it holds no screenshot a tool no longer shows.
   *
   * @returns The screenshot, or null where none was taken: no level is open, or it failed; a call while one is being
   *   taken answers that one's.
   */
  @KeybindCommand(TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerScreenshotService) => service.canTake,
  })
  @ExclusiveFlow()
  public *take(): TFlow<Nullable<ILevelViewerScreenshot>> {
    const level: Nullable<string> = this.snapshotService.level;

    if (!this.canTake || !level) {
      return null;
    }

    const isTimed: boolean = this.renderSettingsService.renderSettings.isGpuTimed;
    // Timed from now, so its reports come in while the screenshot is read back.
    const releaseTiming: () => void = this.renderSettingsService.holdGpuTiming();
    const timedFrame: ITimedFrameWait = isTimed
      ? { frame: Promise.resolve(this.renderService.frame), stop: noop }
      : this.waitForTimedFrame();
    const build: Promise<Nullable<BuildInfo>> = this.snapshotService.readBuild();

    this.isTaking = true;
    this.failure = null;

    try {
      const path: string = yield* call(levelsCommands.findScreenshotPath(toLevelViewerFileStem(level, new Date())));
      // The view as it stands for the frame asked for next, before the camera moves on.
      const view: ILevelViewerSnapshot = this.snapshotService.describe(null);

      if (!(yield* call(this.renderService.saveScreenshot(path)))) {
        throw new Error("The level viewer has no viewport attached");
      }

      const shot: ILevelViewerScreenshot = yield* call(this.read(path));

      this.replace(shot);
      this.log.info("Took a screenshot:", path);

      try {
        const [built, frame] = yield* all([build, timedFrame.frame]);
        const { format, ...described } = view;

        yield* this.writeReport(shot, { format, ...(built ? { build: built } : {}), ...described }, frame, !isTimed);
      } catch (error: unknown) {
        this.fail("write the screenshot's report", error);
      }

      return this.shot;
    } catch (error: unknown) {
      this.fail("take a screenshot", error);

      return null;
    } finally {
      timedFrame.stop();
      releaseTiming();
      this.isTaking = false;
    }
  }

  /** Deletes the last screenshot and its report for good. */
  @ExclusiveFlow()
  public *remove(): TFlow {
    yield* this.attempt("delete the screenshot", (shot: ILevelViewerScreenshot) => this.delete(shot));
  }

  /** Shows the last screenshot selected in its folder. */
  @ExclusiveFlow()
  public *reveal(): TFlow {
    yield* this.attempt("show the screenshot", (shot: ILevelViewerScreenshot) =>
      call(systemCommands.revealPath(shot.path))
    );
  }

  /** Shows the last screenshot's report selected in its folder, once it is written. */
  @ExclusiveFlow()
  public *revealReport(): TFlow {
    yield* this.attempt("show the screenshot's report", ({ reportPath }: ILevelViewerScreenshot) =>
      call(reportPath ? systemCommands.revealPath(reportPath) : Promise.resolve())
    );
  }

  /** Puts the last screenshot's path on the clipboard. */
  @ExclusiveFlow()
  public *copyPath(): TFlow {
    yield* this.attempt("copy the screenshot's path", (shot: ILevelViewerScreenshot) =>
      call(navigator.clipboard.writeText(shot.path))
    );
  }

  /** Puts the last screenshot's image on the clipboard. */
  @ExclusiveFlow()
  public *copyImage(): TFlow {
    yield* this.attempt("copy the screenshot", (shot: ILevelViewerScreenshot) => call(copyLevelViewerImage(shot)));
  }

  /** Copies the last screenshot where a save dialog answers. */
  @ExclusiveFlow()
  public *saveAs(): TFlow {
    yield* this.attempt("save the screenshot", (shot: ILevelViewerScreenshot) => this.saveCopy(shot));
  }

  /** Does `action` with the last screenshot, if any; a failure is logged and kept as `failure`, as `what` says. */
  private *attempt(what: string, action: (shot: ILevelViewerScreenshot) => TFlow<unknown>): TFlow {
    if (!this.shot) {
      return;
    }

    try {
      yield* action(this.shot);
    } catch (error: unknown) {
      this.fail(what, error);
    }
  }

  /** Deletes `shot` and its report, and lets it go. */
  private *delete(shot: ILevelViewerScreenshot): TFlow {
    yield* call(levelsCommands.deleteScreenshot(shot.path));

    this.log.info("Deleted a screenshot:", shot.path);
    this.replace(null);
  }

  /** Copies `shot` where a save dialog answers, if it answers. */
  private *saveCopy(shot: ILevelViewerScreenshot): TFlow {
    const path: Nullable<string> = yield* call(
      save({
        defaultPath: shot.name,
        filters: [{ extensions: [EXrayExtension.PNG], name: "PNG image" }],
        title: "Save screenshot as",
      })
    );

    if (path) {
      yield* call(levelsCommands.copyScreenshot(shot.path, path));
      this.log.info("Saved a screenshot as:", path);
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

    return { ...image, frame: null, reportPath: null };
  }

  /**
   * Writes a screenshot's report beside it and keeps its frame's cost with it, unless it was deleted or replaced
   * meanwhile.
   *
   * @param shot - The screenshot.
   * @param view - The view it shows.
   * @param frame - What its frame cost.
   * @param isTimingSwitched - Whether the GPU timing was switched on for it alone.
   */
  private *writeReport(
    shot: ILevelViewerScreenshot,
    view: ILevelViewerSnapshot,
    frame: RenderFrameReport,
    isTimingSwitched: boolean
  ): TFlow {
    if (this.shot !== shot) {
      return;
    }

    const report: ILevelViewerScreenshotReport = {
      ...view,
      screenshot: { file: shot.name, height: shot.height, isTimingSwitched, width: shot.width },
      frame,
    };

    const reportPath: string = yield* call(
      levelsCommands.writeScreenshotReport(shot.path, serializeLevelViewerSnapshot(report))
    );

    if (this.shot === shot) {
      this.shot = { ...shot, frame, reportPath };
    }
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

  /**
   * The viewport's frame report once its passes are timed, or the latest one should no timed report come in time; or
   * the latest one at once where the wait is stopped, for a screenshot that ended before it came.
   */
  private waitForTimedFrame(): ITimedFrameWait {
    let stop: () => void = noop;

    const frame: Promise<RenderFrameReport> = new Promise((resolve: (frame: RenderFrameReport) => void) => {
      let timed: number = 0;

      const finish = (): void => {
        clearTimeout(timeout);
        dispose();
        resolve(this.renderService.frame);
      };
      const timeout: ReturnType<typeof setTimeout> = setTimeout(finish, TIMED_REPORTS_TIMEOUT);
      const dispose: () => void = reaction(
        () => this.renderService.frame,
        (report: RenderFrameReport) => {
          timed = report.isGpuTimed ? timed + 1 : 0;

          if (timed >= TIMED_REPORTS) {
            finish();
          }
        }
      );

      stop = finish;
    });

    return { frame, stop };
  }
}
