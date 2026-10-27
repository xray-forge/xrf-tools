import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";

import { ELevelCaptureMode, LEVEL_CAPTURE_NAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import { askLevelCaptureExportTarget } from "@/applications/level-viewer/lib/capture/level-capture-export-target";
import { TLevelCapture } from "@/applications/level-viewer/lib/capture/level-captured";
import { toLevelCubeFaceSize } from "@/applications/level-viewer/lib/cube/level-cube-capture";
import {
  DEFAULT_LEVEL_CUBE_EXPORT,
  ILevelCubeExportSettings,
  toCubeExportRequest,
} from "@/applications/level-viewer/lib/cube/level-cube-export";
import { toLevelViewerLevelName } from "@/applications/level-viewer/lib/files/level-viewer-file-stem";
import { copyLevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import {
  DEFAULT_LEVEL_MAP_ANNOTATIONS,
  ILevelMapAnnotations,
  toMapAnnotationsWriteRequest,
} from "@/applications/level-viewer/lib/map/level-map-annotations";
import {
  DEFAULT_LEVEL_MAP_EXPORT,
  ILevelMapExportSettings,
  toLevelMapExportSettings,
  toMapExportRequest,
} from "@/applications/level-viewer/lib/map/level-map-export";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { noop } from "@/lib/callbacks/noop";

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

  /** Where the last export was written, its first file for six, until another capture is shown. */
  @RefObservable()
  public exported: Nullable<string> = null;

  @RefObservable()
  public isExporting: boolean = false;

  /** Why the last thing done with the capture failed, until another capture is shown or exported. */
  @RefObservable()
  public failure: Nullable<string> = null;

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

  /**
   * Shows `captured`, or nothing, letting what it replaces go, with the export written of it and why the last thing
   * done with it failed.
   *
   * @param captured - What was captured, or null for nothing.
   */
  @BoundAction()
  public show(captured: Nullable<TLevelCapture>): void {
    if (this.captured) {
      URL.revokeObjectURL(this.captured.image.url);
    }

    this.captured = captured;
    this.exported = null;
    this.failure = null;
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

    try {
      const { value } = yield* call(writing);

      if (this.captured === captured) {
        this.showAnnotations(URL.createObjectURL(new Blob([value], { type: "image/svg+xml" })));
      }
    } catch (error: unknown) {
      this.fail("draw the map's annotations", error);
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

    const { mode, image } = captured;
    const name: string = LEVEL_CAPTURE_NAMES[mode];
    const level: Nullable<string> = toLevelViewerLevelName(this.loadService.level.value?.selected.value.source);
    const target: Nullable<string> = yield* call(
      askLevelCaptureExportTarget(mode, level ? getPathName(level) : "level", {
        cube: this.cubeExport,
        map: this.mapExport,
      })
    );

    if (!target || this.captured !== captured) {
      return;
    }

    this.isExporting = true;
    this.failure = null;

    try {
      let exported: string = target;

      if (mode === ELevelCaptureMode.MAP) {
        const request = toMapExportRequest(this.mapExport, image.path, target, Math.max(image.width, image.height));

        // The annotations drawn last are on disk before the export reads them.
        yield* call(this.annotationsWritten);
        yield* call(levelsCommands.exportMap(request));
      } else {
        const request = toCubeExportRequest(this.cubeExport, image.path, target, toLevelCubeFaceSize(image.width));
        const written: Array<string> = yield* call(levelsCommands.exportCube(request));

        exported = written[0] ?? target;
      }

      if (this.captured === captured) {
        this.exported = exported;
      }

      this.log.info(`Exported a ${name}:`, target);
    } catch (error: unknown) {
      this.fail(`export the ${name}`, error);
    } finally {
      this.isExporting = false;
    }
  }

  /** Deletes what was captured for good, with its annotations once they are written; nothing while it is exported. */
  public async remove(): Promise<void> {
    if (this.isExporting) {
      return;
    }

    await this.attempt("delete the capture", async ({ image }: TLevelCapture) => {
      await this.annotationsWritten;
      await levelsCommands.deleteScreenshot(image.path);
      this.log.info("Deleted a capture:", image.path);
      this.show(null);
    });
  }

  /** Shows what was captured selected in its folder. */
  public reveal(): Promise<void> {
    return this.attempt("show the capture", ({ image }: TLevelCapture) => systemCommands.revealPath(image.path));
  }

  /** Shows the last export selected in its folder. */
  public async revealExport(): Promise<void> {
    if (this.exported) {
      await systemCommands.revealPath(this.exported).catch((error: unknown) => this.fail("show the export", error));
    }
  }

  /** Puts the capture's path on the clipboard. */
  public copyPath(): Promise<void> {
    return this.attempt("copy the capture's path", ({ image }: TLevelCapture) =>
      navigator.clipboard.writeText(image.path)
    );
  }

  /** Puts the capture's image on the clipboard. */
  public copyImage(): Promise<void> {
    return this.attempt("copy the capture", ({ image }: TLevelCapture) => copyLevelViewerImage(image));
  }

  /** Does `action` with what was captured, if anything; a failure is kept as `failure`, as `what` says. */
  private async attempt(what: string, action: (captured: TLevelCapture) => Promise<unknown>): Promise<void> {
    if (this.captured) {
      await action(this.captured).catch((error: unknown) => this.fail(what, error));
    }
  }

  @BoundAction()
  private fail(what: string, error: unknown): void {
    this.log.error(`Failed to ${what}:`, error);
    this.failure = `Failed to ${what}: ${error instanceof Error ? error.message : String(error)}`;
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
