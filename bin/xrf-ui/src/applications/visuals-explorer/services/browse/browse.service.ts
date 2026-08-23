import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import { assetsCommands } from "@/core/bindings/commands/assets";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { AssetWorldSpec } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * The world being browsed, and every visual in it.
 *
 * Separate from the service that owns the open model because the two have different lifetimes: a root outlives the
 * dozens of models opened under it, and a model can be open with no root at all.
 */
@Injectable()
export class VisualsBrowseService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The world being browsed, or null when a single model was opened directly. */
  @Observable()
  public world: Nullable<AssetWorldSpec> = null;

  @Observable()
  public visuals: Loadable<Array<XrayAsset>> = createLoadable([]);

  /**
   * @returns Whether a world is open, which is what publishes the tree panel.
   */
  @Computed()
  public get isBrowsing(): boolean {
    return this.world !== null;
  }

  /**
   * @returns The root being browsed, for the surfaces that name it to the user.
   */
  @Computed()
  public get root(): Nullable<string> {
    return this.world?.roots[0] ?? null;
  }

  /**
   * @returns The roots an open searches ahead of the project's own.
   */
  @Computed()
  public get roots(): Array<string> {
    return this.world?.roots ?? [];
  }

  public constructor() {
    makeObservable(this);
  }

  /**
   * Restore whatever world the backend is still browsing.
   *
   * A reload loses the tree but not the session, and coming back to an empty panel beside a model that is still open
   * reads as a failure rather than a fresh start.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const world: Nullable<AssetWorldSpec> = await visualsCommands.getBrowse();

      if (world) {
        this.log.info("Restoring browsed world:", world.roots.join(", "));

        await this.list(world);
      }
    } catch (error) {
      this.log.error("Failed to restore browsed world:", error);
    }
  }

  /**
   * Drop the browsed world on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating and releasing the project");

    runInAction(() => {
      this.world = null;
      this.visuals = createLoadable([]);
    });

    releaseEditorProject(visualsCommands.closeBrowse);
  }

  /**
   * Browse a root and list every visual in it.
   *
   * @param root - Filesystem path of the directory or installation to browse.
   */
  @BoundAction()
  public async openRoot(root: string): Promise<void> {
    const world: AssetWorldSpec = { asset: null, roots: [root] };

    this.log.info("Browsing root:", root);

    await visualsCommands.openBrowse(world);
    await this.list(world);
  }

  /** Stop browsing, leaving whatever model is open on screen. */
  @BoundAction()
  public async close(): Promise<void> {
    runInAction(() => {
      this.world = null;
      this.visuals = createLoadable([]);
    });

    try {
      await visualsCommands.closeBrowse();
    } catch (error) {
      this.log.error("Failed to close browsed world:", error);
    }
  }

  /**
   * Lists a world and puts it on screen.
   *
   * @param world - World to list, already recorded as the browsed one.
   */
  private async list(world: AssetWorldSpec): Promise<void> {
    runInAction(() => {
      this.world = world;
      this.visuals = this.visuals.asLoading();
    });

    try {
      const visuals: Array<XrayAsset> = await assetsCommands.listAssets(world, "ogf");

      runInAction(() => {
        this.visuals = this.visuals.asReady(visuals);
      });

      this.log.info(`Listed ${visuals.length} visuals in:`, world.roots.join(", "));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list visuals:", transformed);

      runInAction(() => {
        this.visuals = this.visuals.asFailed(transformed, []);
      });
    }
  }
}
