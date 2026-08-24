import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { assetsCommands } from "@/core/bindings/commands/assets";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { XrayAsset, XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
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

  /** What is being browsed, or null when a single model was opened directly. */
  @Observable()
  public browsed: Nullable<XrayRoots> = null;

  @Observable()
  public visuals: Loadable<Array<XrayAsset>> = createLoadable([]);

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
    return this.browsed?.roots[0]?.path ?? null;
  }

  /**
   * @returns The paths an open searches ahead of the project's own.
   */
  @Computed()
  public get rootPaths(): Array<string> {
    return this.browsed?.roots.map((root: XrayRoot) => root.path) ?? [];
  }

  /**
   * Restore whatever roots the backend is still browsing.
   *
   * A reload loses the tree but not the session, and coming back to an empty panel beside a model that is still open
   * reads as a failure rather than a fresh start.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const roots: Nullable<XrayRoots> = await visualsCommands.getBrowse();

      if (roots) {
        this.log.info("Restoring browsed roots:", describeRoots(roots));

        // Through the lane rather than around it, so a root the user picks while this is still restoring wins.
        await flowResult(this.restore(roots));
      }
    } catch (error) {
      this.log.error("Failed to restore browsed roots:", error);
    }
  }

  /**
   * Drop the browsed roots on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating and releasing the project");

    runInAction(() => {
      this.browsed = null;
      this.visuals = createLoadable([]);
    });

    releaseEditorProject(visualsCommands.closeBrowse);
  }

  /**
   * Browse a root and list every visual in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   */
  @LatestFlow("visuals")
  public *openRoot(root: string): TFlow {
    const roots: XrayRoots = createRoots([root]);

    this.log.info("Browsing root:", root);

    yield* call(visualsCommands.openBrowse(roots));
    yield* this.list(roots);
  }

  /** Stop browsing, leaving whatever model is open on screen. */
  @LatestFlow("visuals")
  public *close(): TFlow {
    this.browsed = null;
    this.visuals = createLoadable([]);

    try {
      yield* call(visualsCommands.closeBrowse());
    } catch (error) {
      this.log.error("Failed to close browsed roots:", error);
    }
  }

  /**
   * Puts an already browsed roots back on screen, for a session the backend still holds.
   *
   * @param roots - Roots the backend reported as browsed.
   */
  @LatestFlow("visuals")
  private *restore(roots: XrayRoots): TFlow {
    yield* this.list(roots);
  }

  /**
   * Lists roots and puts the result on screen.
   *
   * A generator so a listing the user has moved past is abandoned rather than published: the write below the yield
   * cannot run once another root has taken the lane.
   *
   * @param roots - Roots to list, already recorded as the browsed one.
   */
  private *list(roots: XrayRoots): TFlow {
    this.browsed = roots;
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
