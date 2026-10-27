import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { TFlow } from "@xrf/flow";
import { assetsCommands } from "@xrf/ipc/commands/assets";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { EXrayAssetType, XrayAsset, XrayRoot, XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { createRoots, describeRoots } from "@/core/assets/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * The roots being browsed, and every visual in them.
 */
@Injectable()
export class VisualsBrowseService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What is being browsed, or nothing when a single model was opened directly. */
  public readonly roots: SessionRootResource<XrayRoots> = new SessionRootResource(this.log, {
    subject: "visual roots",
    close: visualsCommands.closeBrowse,
    restore: visualsCommands.restoreBrowse,
    adopted: (snapshot: SessionSnapshot<XrayRoots>) => this.list(snapshot.value),
  });

  /** Every visual the browsed roots hold, listed afresh for each root adopted. */
  public readonly visuals: CommandOperation<Array<XrayAsset>> = new CommandOperation(this.log);

  /**
   * @returns Whether anything is open, which is what publishes the tree panel.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.roots.value !== null;
  }

  /**
   * @returns Every root being browsed as one line, or null when a single model was opened with no root at all.
   */
  @Computed()
  public get rootsLabel(): Nullable<string> {
    return this.roots.value ? describeRoots(this.roots.value) : null;
  }

  /**
   * @returns The paths an open searches ahead of the project's own.
   */
  @Computed()
  public get rootPaths(): Array<string> {
    return this.roots.value?.roots.map((root: XrayRoot) => root.path) ?? [];
  }

  /**
   * Restore whatever roots the backend is still browsing.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.roots.restore());
  }

  /**
   * Browse a root and list every visual in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   * @param assetRoot - A further tree searched behind it, or null to read only the root itself.
   * @returns Whether the root opened; a listing that failed after it still counts, and shows its own failure.
   */
  @BoundAction()
  public openRoot(root: string, assetRoot: Nullable<string> = null): Promise<boolean> {
    // Both go to the backend, which keeps them for the session: a reload restores the pair rather than the first of
    // them, so a later read searches what the open searched.
    const roots: XrayRoots = createRoots([root, assetRoot]);

    return flowResult(this.roots.open(describeRoots(roots), visualsCommands.openBrowse, roots));
  }

  /** Stop browsing, leaving whatever model is open on screen. */
  @BoundAction()
  public async close(): Promise<void> {
    if (await flowResult(this.roots.close())) {
      this.visuals.reset();
    }
  }

  /**
   * Lists the roots adopted, starting from nothing so a failed listing never shows another root's visuals.
   *
   * @param roots - Roots the backend committed.
   */
  private *list(roots: XrayRoots): TFlow {
    this.visuals.reset();

    yield* this.visuals.run({
      name: "List visuals",
      subject: describeRoots(roots),
      command: () => assetsCommands.listAssets(roots, EXrayAssetType.OGF),
      failure: "Could not list the visuals",
    });
  }
}
