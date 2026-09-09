import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { ConfigsProjectDescriptor } from "@/core/bindings/types/xrf-app";
import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * Which configs project is open, and what it holds.
 *
 * The session is the backend's, not this object's: everything read later is addressed by the id the open answered
 * with, so a reopen invalidates every document and resolution the frontend is holding without either side tracking
 * them individually.
 */
@Injectable()
export class ConfigsProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Whether the backend has been asked what it still has open. */
  @Observable()
  public isReady: boolean = false;

  /** The open project, or null when nothing is open. */
  @Observable()
  public project: Loadable<Nullable<ConfigsProjectDescriptor>> = Loadable.idle(null);

  /**
   * @returns Whether a project is open, which is what puts the tree on screen.
   */
  @Computed()
  public get isOpen(): boolean {
    return this.project.value !== null;
  }

  /**
   * @returns Every config the project holds, or nothing when none is open.
   */
  @Computed()
  public get files(): Array<LtxInventoryFile> {
    return this.project.value?.inventory.files ?? [];
  }

  /**
   * @returns The identity every read must be addressed by, or null when nothing is open.
   */
  @Computed()
  public get sessionId(): Nullable<string> {
    return this.project.value?.sessionId ?? null;
  }

  /**
   * Restore whatever project the backend still has open.
   *
   * A reload loses the tree but not the session, so the descriptor comes back with its inventory and the screen
   * returns to where it was rather than to the picker.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const descriptor: Nullable<ConfigsProjectDescriptor> = await configsCommands.getProject();

      if (descriptor) {
        this.log.info("Restoring opened configs project:", descriptor.root);

        // Through the lane rather than around it, so a project the user opens while this is still restoring wins.
        await flowResult(this.restore(descriptor));

        return;
      }
    } catch (error) {
      this.log.error("Failed to restore the opened configs project:", error);
    }

    runInAction(() => {
      // Nothing to restore: the picker is the answer. A restore still in flight owns the announcement, so this speaks
      // only when nothing is arriving.
      if (!this.project.isLoading) {
        this.isReady = true;
      }
    });
  }

  /**
   * Drop the project on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating and releasing the opened configs project");

    runInAction(() => {
      this.project = this.project.asIdle(null);
    });

    releaseEditorProject(configsCommands.closeProject);
  }

  /**
   * Open a configs tree for browsing.
   *
   * @param root - Filesystem path of a configs directory or a game installation root.
   * @param isDltx - Whether to resolve with the Monolith/Anomaly patch dialect.
   * @param prefix - Logical subtree holding the configs, or null for the whole root.
   */
  @LatestFlow("project")
  public *open(root: string, isDltx: boolean, prefix: Nullable<string> = null): TFlow {
    this.project = this.project.asLoading();

    try {
      const descriptor: ConfigsProjectDescriptor = yield* call(
        configsCommands.openProject({
          isDltx,
          prefix,
          roots: createRoots([root]),
        })
      );

      this.log.info("Opened configs project:", descriptor.root, descriptor.inventory.files.length, "configs");

      this.project = this.project.asReady(descriptor);
    } catch (error) {
      this.log.error("Failed to open the configs project:", error);

      this.project = this.project.asFailed(transformError(error));
    } finally {
      this.isReady = true;
    }
  }

  /**
   * Close the project and go back to the picker.
   */
  @LatestFlow("project")
  public *close(): TFlow {
    this.project = this.project.asIdle(null);

    try {
      yield* call(configsCommands.closeProject());
    } catch (error) {
      this.log.error("Failed to close the configs project:", error);
    }
  }

  /**
   * Adopt a descriptor the backend already had open.
   *
   * @param descriptor - What the backend reported it is holding.
   */
  @LatestFlow("project")
  private *restore(descriptor: ConfigsProjectDescriptor): TFlow {
    this.project = this.project.asReady(descriptor);
    this.isReady = true;

    yield;
  }
}
