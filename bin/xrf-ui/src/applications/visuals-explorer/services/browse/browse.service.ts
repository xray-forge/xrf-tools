import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { assetsCommands } from "@/core/ipc/commands/assets";
import { visualsCommands } from "@/core/ipc/commands/visuals";
import { Session } from "@/core/ipc/session";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { EXrayAssetType, XrayAsset, XrayRoot, XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * The roots being browsed, and every visual in them.
 */
@Injectable()
export class VisualsBrowseService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(visualsCommands.closeBrowse);

  /** What is being browsed, or null when a single model was opened directly. */
  @Observable()
  private browsed: Nullable<SessionSnapshot<XrayRoots>> = null;

  @Observable()
  public visuals: AsyncState<Array<XrayAsset>> = AsyncState.idle([]);

  /**
   * @returns Whether anything is open, which is what publishes the tree panel.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.browsed !== null;
  }

  /**
   * @returns Every root being browsed as one line, or null when a single model was opened with no root at all.
   */
  @Computed()
  public get rootsLabel(): Nullable<string> {
    return this.browsed ? describeRoots(this.browsed.value) : null;
  }

  /**
   * @returns The paths an open searches ahead of the project's own.
   */
  @Computed()
  public get rootPaths(): Array<string> {
    return this.browsed?.value.roots.map((root: XrayRoot) => root.path) ?? [];
  }

  /**
   * Restore whatever roots the backend is still browsing.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Drop the browsed roots on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();

    runInAction(() => {
      this.browsed = null;
      this.visuals = this.visuals.asIdle([]);
    });
  }

  /**
   * Browse a root and list every visual in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   * @param assetRoot - A further tree searched behind it, or null to read only the root itself.
   */
  @LatestFlow("visuals")
  public *openRoot(root: string, assetRoot: Nullable<string> = null): TFlow {
    // Both go to the backend, which keeps them for the session: a reload restores the pair rather than the first of
    // them, so a later read searches what the open searched.
    const roots: XrayRoots = createRoots([root, assetRoot]);

    this.log.info("Browsing root:", root);

    let opened: SessionSnapshot<XrayRoots>;

    try {
      opened = yield* call(this.session.open(visualsCommands.openBrowse, roots));
    } catch (error: unknown) {
      this.log.error("Failed to open visual roots:", describeRoots(roots), error);

      throw error;
    }

    yield* this.list(opened);
  }

  /** Stop browsing, leaving whatever model is open on screen. */
  @LatestFlow("visuals")
  public *close(): TFlow {
    const roots: Nullable<string> = this.rootsLabel;

    try {
      yield* call(this.session.close());

      this.browsed = null;
      this.visuals = this.visuals.asIdle([]);

      if (roots) {
        this.log.info("Visual roots closed:", roots);
      }
    } catch (error) {
      this.log.error("Failed to close browsed roots:", roots, error);
    }
  }

  /**
   * Restores browsed roots unless a user action has taken the browsing flow.
   */
  @ExclusiveFlow("visuals")
  private *restore(): TFlow {
    try {
      const snapshot = yield* call(visualsCommands.getBrowse());

      this.session.adopt(snapshot);

      if (snapshot) {
        this.log.info("Restoring visual roots:", describeRoots(snapshot.value));
        yield* this.list(snapshot);
      }
    } catch (error) {
      this.log.error("Failed to restore browsed roots:", error);
    }
  }

  /**
   * Lists roots and puts the result on screen.
   *
   * @param opened - Roots and identity already committed by the backend.
   */
  private *list(opened: SessionSnapshot<XrayRoots>): TFlow {
    const roots: XrayRoots = opened.value;
    const timer: Timer = new Timer();

    this.browsed = opened;
    this.visuals = this.visuals.asLoading();

    try {
      const visuals: Array<XrayAsset> = yield* call(assetsCommands.listAssets(roots, EXrayAssetType.OGF));

      this.visuals = this.visuals.asReady(visuals);

      this.log.info(`Listed ${visuals.length} visuals:`, describeRoots(roots), "in", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to list visuals:",
        describeRoots(roots),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.visuals = this.visuals.asFailed(transformed, []);
    }
  }
}
