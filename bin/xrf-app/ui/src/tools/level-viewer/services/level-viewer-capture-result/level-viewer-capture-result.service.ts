import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { LevelLoadService } from "@/core/level/services/level-load";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { CommandReporter } from "@/core/operations/lib/command-reporter";
import { noop } from "@/lib/callbacks/noop";
import { ELevelCaptureMode, LEVEL_CAPTURE_NAMES } from "@/tools/level-viewer/lib/capture/level-capture";
import { askLevelCaptureExportTarget } from "@/tools/level-viewer/lib/capture/level-capture-export-target";
import { TLevelCapture } from "@/tools/level-viewer/lib/capture/level-captured";
import { toLevelCubeFaceSize } from "@/tools/level-viewer/lib/cube/level-cube-capture";
import {
  DEFAULT_LEVEL_CUBE_EXPORT,
  ILevelCubeExportSettings,
  toCubeExportRequest,
} from "@/tools/level-viewer/lib/cube/level-cube-export";
import { toLevelViewerFileName, toLevelViewerLevelName } from "@/tools/level-viewer/lib/files/level-viewer-file-stem";
import { copyLevelViewerImage } from "@/tools/level-viewer/lib/files/level-viewer-image";
import {
  DEFAULT_LEVEL_MAP_ANNOTATIONS,
  ILevelMapAnnotations,
  toMapAnnotationsWriteRequest,
} from "@/tools/level-viewer/lib/map/level-map-annotations";
import {
  DEFAULT_LEVEL_MAP_EXPORT,
  ILevelMapExportSettings,
  toLevelMapExportSettings,
  toMapExportRequest,
} from "@/tools/level-viewer/lib/map/level-map-export";

/**
 * What the level viewer's capture wrote, while its dialog shows it: the picture, the annotations drawn over a map and
 * written beside it, how it is exported and where it last was, and deleting, revealing and copying it.
 */
@Injectable()
export class LevelViewerCaptureResultService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What was last captured, until it is deleted, left or the dialog closes. */
  @RefObservable()
  public captured: Nullable<TLevelCapture> = null;

  /** What of a map's annotations is drawn over it, and the SVG drawn of them, as an object URL, until it changes. */
  @RefObservable()
  public mapAnnotations: ILevelMapAnnotations = DEFAULT_LEVEL_MAP_ANNOTATIONS;

  @RefObservable()
  public annotationsUrl: Nullable<string> = null;

  /** How the next export of a map and of a cubemap is written. */
  @RefObservable()
  public mapExport: ILevelMapExportSettings = DEFAULT_LEVEL_MAP_EXPORT;

  @RefObservable()
  public cubeExport: ILevelCubeExportSettings = DEFAULT_LEVEL_CUBE_EXPORT;

  /** The capture's export: where the last was written, its first file for six, until another capture is shown. */
  public readonly exported: CommandOperation<string> = new CommandOperation(this.log);

  /** Reports what else is done with the capture: its annotations drawn, deleting, showing and copying it. */
  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  /**
   * The last of the annotations' writes beside the map, each sent once the one before it ended, so the file on disk is
   * the last drawn, which an export reads.
   */
  private annotationsWritten: Promise<unknown> = Promise.resolve();

  public constructor(private readonly loadService: LevelLoadService = inject(LevelLoadService)) {}

  @OnDeactivation()
  public onDeactivation(): void {
    this.show(null);
  }

  /** The backend's opening of the level, which its annotations are drawn from, or null for none. */
  @Computed()
  public get sessionId(): Nullable<string> {
    return this.loadService.level.value?.selected.sessionId ?? null;
  }

  /** Whether the capture is being exported, during which it is neither deleted nor left. */
  @Computed()
  public get isExporting(): boolean {
    return this.exported.isLoading;
  }

  /**
   * Shows `captured`, or nothing, letting what it replaces go with the export written of it: an export still running
   * lands on no other capture.
   *
   * @param captured - What was captured, or null for nothing.
   */
  @BoundAction()
  public show(captured: Nullable<TLevelCapture>): void {
    if (this.captured) {
      URL.revokeObjectURL(this.captured.image.url);
    }

    cancelFlow(this, "exportAs");
    this.captured = captured;
    this.exported.reset();
    this.showAnnotations(null);
  }

  /** Shows nothing and puts every export and annotation setting back, for a dialog opened afresh. */
  @BoundAction()
  public reset(): void {
    this.show(null);
    this.mapExport = DEFAULT_LEVEL_MAP_EXPORT;
    this.cubeExport = DEFAULT_LEVEL_CUBE_EXPORT;
    this.mapAnnotations = DEFAULT_LEVEL_MAP_ANNOTATIONS;
  }

  /** Goes back to the parameters from what was captured, which stays in its folder; nothing while it is exported. */
  @BoundAction()
  public back(): void {
    if (!this.isExporting) {
      this.show(null);
    }
  }

  /**
   * @param annotations - What of a map's annotations is drawn from now on, which draws them again.
   */
  @BoundAction()
  public setMapAnnotations(annotations: ILevelMapAnnotations): void {
    this.mapAnnotations = annotations;
    void this.drawAnnotations();
  }

  /** Draws the annotations asked for over the map shown and writes them beside it, then shows them while it is. */
  @LatestFlow()
  public *drawAnnotations(): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;
    const sessionId: Nullable<string> = this.sessionId;

    if (captured?.mode !== ELevelCaptureMode.MAP || !sessionId) {
      return;
    }

    const request = toMapAnnotationsWriteRequest(this.mapAnnotations, captured.image.path, captured.area);
    const writing: Promise<SessionSnapshot<string>> = this.annotationsWritten.then(() =>
      levelsCommands.writeMapAnnotations(sessionId, request)
    );

    this.annotationsWritten = writing.catch(noop);

    const { result } = yield* this.reporter.run({
      name: "Draw map annotations",
      subject: captured.image.path,
      command: () => writing,
      failure: "Could not draw the map's annotations",
    });

    if (result && this.captured === captured) {
      this.showAnnotations(URL.createObjectURL(new Blob([result.value], { type: "image/svg+xml" })));
    }
  }

  /**
   * @param patch - What of the map's export changes, kept to an export the backend writes.
   */
  @BoundAction()
  public setMapExport(patch: Partial<ILevelMapExportSettings>): void {
    this.mapExport = toLevelMapExportSettings(this.mapExport, patch);
  }

  /**
   * @param patch - What of the cubemap's export changes.
   */
  @BoundAction()
  public setCubeExport(patch: Partial<ILevelCubeExportSettings>): void {
    this.cubeExport = { ...this.cubeExport, ...patch };
  }

  /**
   * Writes what was captured again as its export settings ask, where a save dialog answers: a file, or the folder a
   * cubemap's six faces are written into.
   */
  @ExclusiveFlow()
  public *exportAs(): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;

    if (!captured) {
      return;
    }

    const name: string = LEVEL_CAPTURE_NAMES[captured.mode];
    const level: Nullable<string> = toLevelViewerLevelName(this.loadService.level.value?.selected.value.source);
    const { result: target } = yield* this.reporter.run({
      name: `Choose where to export the ${name}`,
      command: () =>
        askLevelCaptureExportTarget(captured.mode, toLevelViewerFileName(level), {
          cube: this.cubeExport,
          map: this.mapExport,
        }),
      failure: `Could not export the ${name}`,
    });

    if (!target || this.captured !== captured) {
      return;
    }

    yield* this.exported.run({
      name: `Export ${name}`,
      subject: target,
      command: () => this.writeExport(captured, target),
      failure: `Could not export the ${name}`,
    });
  }

  /** Deletes what was captured for good, with its annotations once they are written; nothing while it is exported. */
  @ExclusiveFlow()
  public *remove(): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;

    if (!captured || this.isExporting) {
      return;
    }

    const { error } = yield* this.reporter.run({
      name: "Delete capture",
      subject: captured.image.path,
      command: () => this.delete(captured),
      failure: "Could not delete the capture",
    });

    if (!error) {
      this.show(null);
    }
  }

  /** Shows what was captured selected in its folder. */
  @ExclusiveFlow()
  public *reveal(): TFlow {
    yield* this.attempt("Show capture", "Could not show the capture", ({ image }) =>
      systemCommands.revealPath(image.path)
    );
  }

  /** Shows the last export selected in its folder. */
  @ExclusiveFlow()
  public *revealExport(): TFlow {
    const exported: Nullable<string> = this.exported.value;

    if (exported) {
      yield* this.reporter.run({
        name: "Show export",
        subject: exported,
        command: () => systemCommands.revealPath(exported),
        failure: "Could not show the export",
      });
    }
  }

  /** Puts the capture's path on the clipboard. */
  @ExclusiveFlow()
  public *copyPath(): TFlow {
    yield* this.attempt("Copy capture path", "Could not copy the capture's path", ({ image }) =>
      navigator.clipboard.writeText(image.path)
    );
  }

  /** Puts the capture's image on the clipboard. */
  @ExclusiveFlow()
  public *copyImage(): TFlow {
    yield* this.attempt("Copy capture", "Could not copy the capture", ({ image }) => copyLevelViewerImage(image));
  }

  /**
   * Does something with what was captured, if anything, reported as one command.
   *
   * @param name - What it does, as its log lines name it.
   * @param failure - The failure notice's title.
   * @param action - Does it with the capture.
   */
  private *attempt(name: string, failure: string, action: (captured: TLevelCapture) => Promise<unknown>): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;

    if (captured) {
      yield* this.reporter.run({ name, subject: captured.image.path, command: () => action(captured), failure });
    }
  }

  /**
   * Writes `captured` again as its export settings ask.
   *
   * @param captured - What was captured.
   * @param target - The file, or a cubemap's folder for six faces, the save dialog answered.
   * @returns Where it was written, its first file for six.
   */
  private async writeExport(captured: TLevelCapture, target: string): Promise<string> {
    const { mode, image } = captured;

    if (mode === ELevelCaptureMode.MAP) {
      const request = toMapExportRequest(this.mapExport, image.path, target, Math.max(image.width, image.height));

      // The annotations drawn last are on disk before the export reads them.
      await this.annotationsWritten;
      await levelsCommands.exportMap(request);

      return target;
    }

    const request = toCubeExportRequest(this.cubeExport, image.path, target, toLevelCubeFaceSize(image.width));
    const written: Array<string> = await levelsCommands.exportCube(request);

    return written[0] ?? target;
  }

  /** Deletes `captured` once the annotations drawn over it are written. */
  private async delete({ image }: TLevelCapture): Promise<void> {
    await this.annotationsWritten;
    await levelsCommands.deleteScreenshot(image.path);
  }

  /** Shows the annotations drawn at `url`, or none, letting the ones they replace go. */
  @BoundAction()
  private showAnnotations(url: Nullable<string>): void {
    if (this.annotationsUrl) {
      URL.revokeObjectURL(this.annotationsUrl);
    }

    this.annotationsUrl = url;
  }
}
