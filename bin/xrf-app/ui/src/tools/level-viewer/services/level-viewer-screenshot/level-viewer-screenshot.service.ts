import { save } from "@tauri-apps/plugin-dialog";
import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, reaction, RefObservable } from "@wirestate/mobx";
import { cancelFlow, ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";
import { RenderFrameReport } from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { LevelRenderService } from "@/core/level/services/level-render";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { noop } from "@/lib/callbacks/noop";
import { TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND } from "@/tools/level-viewer/commands";
import { toLevelViewerFileStem } from "@/tools/level-viewer/lib/files/level-viewer-file-stem";
import { copyLevelViewerImage, readLevelViewerImage } from "@/tools/level-viewer/lib/files/level-viewer-image";
import {
  ILevelViewerScreenshot,
  ILevelViewerScreenshotReport,
  ILevelViewerScreenshotReportFile,
} from "@/tools/level-viewer/lib/screenshot/level-viewer-screenshot";
import {
  ILevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/tools/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

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

  /** The last screenshot taken, until it is deleted; the one before it shown while another is taken. */
  public readonly shot: CommandOperation<ILevelViewerScreenshot> = new CommandOperation(this.log);

  /** The last screenshot's report, once written beside it. */
  public readonly report: CommandOperation<ILevelViewerScreenshotReportFile> = new CommandOperation(this.log);

  /** Whether the last screenshot is shown whole, in its dialog. */
  @RefObservable()
  public isShown: boolean = false;

  /** Reports what is done with a screenshot taken: deleting, showing, copying and saving it. */
  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  public constructor(
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly snapshotService: LevelViewerSnapshotService = inject(LevelViewerSnapshotService),
    private readonly renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {}

  @OnDeactivation()
  public onDeactivation(): void {
    this.forget();
  }

  /** Whether a screenshot is being taken or its report written, during which another is not taken. */
  @Computed()
  public get isTaking(): boolean {
    return this.shot.isLoading || this.report.isLoading;
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
    this.isShown = isShown && this.shot.value !== null;
  }

  /**
   * Saves the viewport's next finished frame to the screenshots folder, then its report once its frame's cost is read;
   * the passes are timed on the GPU meanwhile, the timing setting left as it is. A failure of either is noticed apart,
   * and a screenshot whose report failed is kept.
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
    const previous: Nullable<ILevelViewerScreenshot> = this.shot.value;

    try {
      const taken: TCommandCompletion<ILevelViewerScreenshot> = yield* this.shot.run({
        name: "Take screenshot",
        subject: level,
        command: () => this.save(toLevelViewerFileStem(level, new Date())),
        failure: "Could not take a screenshot",
      });

      if (taken.error) {
        return null;
      }

      if (previous) {
        URL.revokeObjectURL(previous.url);
      }

      this.report.reset();

      yield* this.report.run({
        name: "Write screenshot report",
        subject: taken.result.path,
        command: () => this.writeReport(taken.result, build, timedFrame.frame, !isTimed),
        failure: "Could not write the screenshot's report",
      });

      return taken.result;
    } finally {
      timedFrame.stop();
      releaseTiming();
    }
  }

  /** Deletes the last screenshot and its report for good. */
  @ExclusiveFlow()
  public *remove(): TFlow {
    const shot: Nullable<ILevelViewerScreenshot> = this.shot.value;

    if (!shot) {
      return;
    }

    const completion: TCommandCompletion<unknown> = yield* this.reporter.run({
      name: "Delete screenshot",
      subject: shot.path,
      command: () => levelsCommands.deleteScreenshot(shot.path),
      failure: "Could not delete the screenshot",
    });

    if (!completion.error) {
      this.forget();
    }
  }

  /** Shows the last screenshot selected in its folder. */
  @ExclusiveFlow()
  public *reveal(): TFlow {
    yield* this.attempt("Show screenshot", "Could not show the screenshot", (shot) =>
      systemCommands.revealPath(shot.path)
    );
  }

  /** Shows the last screenshot's report selected in its folder, once it is written. */
  @ExclusiveFlow()
  public *revealReport(): TFlow {
    const report: Nullable<ILevelViewerScreenshotReportFile> = this.report.value;

    if (report) {
      yield* this.reporter.run({
        name: "Show screenshot report",
        subject: report.path,
        command: () => systemCommands.revealPath(report.path),
        failure: "Could not show the screenshot's report",
      });
    }
  }

  /** Puts the last screenshot's path on the clipboard. */
  @ExclusiveFlow()
  public *copyPath(): TFlow {
    yield* this.attempt("Copy screenshot path", "Could not copy the screenshot's path", (shot) =>
      navigator.clipboard.writeText(shot.path)
    );
  }

  /** Puts the last screenshot's image on the clipboard. */
  @ExclusiveFlow()
  public *copyImage(): TFlow {
    yield* this.attempt("Copy screenshot", "Could not copy the screenshot", (shot) => copyLevelViewerImage(shot));
  }

  /** Copies the last screenshot where a save dialog answers. */
  @ExclusiveFlow()
  public *saveAs(): TFlow {
    yield* this.attempt("Save screenshot as", "Could not save the screenshot", (shot) => this.saveCopy(shot));
  }

  /**
   * Does something with the last screenshot, if any, reported as one command.
   *
   * @param name - What it does, as its log lines name it.
   * @param failure - The failure notice's title.
   * @param action - Does it with the screenshot.
   */
  private *attempt(name: string, failure: string, action: (shot: ILevelViewerScreenshot) => Promise<unknown>): TFlow {
    const shot: Nullable<ILevelViewerScreenshot> = this.shot.value;

    if (shot) {
      yield* this.reporter.run({ name, subject: shot.path, command: () => action(shot), failure });
    }
  }

  /**
   * Saves the viewport's next finished frame under a free name in the screenshots folder and reads it back.
   *
   * @param stem - What the file is named for.
   * @returns The screenshot, with the view as it stood for its frame.
   */
  private async save(stem: string): Promise<ILevelViewerScreenshot> {
    const path: string = await levelsCommands.findScreenshotPath(stem);
    // The view as it stands for the frame asked for next, before the camera moves on.
    const view: ILevelViewerSnapshot = this.snapshotService.describe(null);

    if (!(await this.renderService.saveScreenshot(path))) {
      throw new Error("The level viewer has no viewport attached");
    }

    return { ...(await readLevelViewerImage(path)), view };
  }

  /** Copies `shot` where a save dialog answers, if it answers. */
  private async saveCopy(shot: ILevelViewerScreenshot): Promise<void> {
    const path: Nullable<string> = await save({
      defaultPath: shot.name,
      filters: [{ extensions: [EXrayExtension.PNG], name: "PNG image" }],
      title: "Save screenshot as",
    });

    if (path) {
      await levelsCommands.copyScreenshot(shot.path, path);
      this.log.info("Saved a screenshot as:", path);
    }
  }

  /**
   * Writes a screenshot's report beside it.
   *
   * @param shot - The screenshot.
   * @param build - The build that drew it, once read.
   * @param timedFrame - What its frame cost, once its passes are timed.
   * @param isTimingSwitched - Whether the GPU timing was switched on for it alone.
   * @returns Where the report was written, and the frame's cost it holds.
   */
  private async writeReport(
    shot: ILevelViewerScreenshot,
    build: Promise<Nullable<BuildInfo>>,
    timedFrame: Promise<RenderFrameReport>,
    isTimingSwitched: boolean
  ): Promise<ILevelViewerScreenshotReportFile> {
    const [built, frame] = await Promise.all([build, timedFrame]);
    const { format, ...view } = shot.view;
    const report: ILevelViewerScreenshotReport = {
      format,
      ...(built ? { build: built } : {}),
      ...view,
      screenshot: { file: shot.name, height: shot.height, isTimingSwitched, width: shot.width },
      frame,
    };

    return { frame, path: await levelsCommands.writeScreenshotReport(shot.path, serializeLevelViewerSnapshot(report)) };
  }

  /** Lets the last screenshot and its report go, a take still running included. */
  @BoundAction()
  private forget(): void {
    cancelFlow(this, "take");

    if (this.shot.value) {
      URL.revokeObjectURL(this.shot.value.url);
    }

    this.shot.reset();
    this.report.reset();
    this.isShown = false;
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
