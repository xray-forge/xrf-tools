import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { assetsCommands } from "@/core/bindings/commands/assets";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { XrayAsset, XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { Session } from "@/core/ipc/session";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The roots being browsed, and every visual in them.
 *
 * Separate from the service that owns the open model because the two have different lifetimes: a root outlives the
 * dozens of models opened under it, and a model can be open with no root at all.
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
   * @returns The root being browsed, for the surfaces that name it to the user.
   */
  @Computed()
  public get root(): Nullable<string> {
    return this.browsed?.value.roots[0]?.path ?? null;
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
   *
   * A reload loses the tree but not the session, and coming back to an empty panel beside a model that is still open
   * reads as a failure rather than a fresh start.
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
    this.log.info("Deactivating and releasing the project");

    releaseEditorProject(() => this.session.close(this.browsed?.sessionId));

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

    const opened = yield* call(this.session.open(visualsCommands.openBrowse, roots));

    yield* this.list(opened);
  }

  /** Stop browsing, leaving whatever model is open on screen. */
  @LatestFlow("visuals")
  public *close(): TFlow {
    try {
      yield* call(this.session.close(this.browsed?.sessionId));

      this.browsed = null;
      this.visuals = this.visuals.asIdle([]);
    } catch (error) {
      this.log.error("Failed to close browsed roots:", error);
    }
  }

  /**
   * Restores browsed roots unless a user action has taken the browsing flow.
   */
  @ExclusiveFlow("visuals")
  private *restore(): TFlow {
    try {
      const snapshot = yield* call(visualsCommands.getBrowse());

      if (snapshot) {
        yield* this.list(snapshot);
      }
    } catch (error) {
      this.log.error("Failed to restore browsed roots:", error);
    }
  }

  /**
   * Lists roots and puts the result on screen.
   *
   * A generator so a listing the user has moved past is abandoned rather than published: the write below the yield
   * cannot run once another root has taken the lane.
   *
   * @param opened - Roots and identity already committed by the backend.
   */
  private *list(opened: SessionSnapshot<XrayRoots>): TFlow {
    const roots: XrayRoots = opened.value;

    this.browsed = opened;
    this.visuals = this.visuals.asLoading();

    try {
      const visuals: Array<XrayAsset> = yield* call(assetsCommands.listAssets(roots, "ogf"));

      this.visuals = this.visuals.asReady(visuals);

      this.log.info(`Listed ${visuals.length} visuals in:`, describeRoots(roots));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list visuals:", transformed);

      this.visuals = this.visuals.asFailed(transformed, []);
    }
  }
}
