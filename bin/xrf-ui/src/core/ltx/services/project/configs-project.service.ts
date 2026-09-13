import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { configsCommands } from "@/core/ipc/commands/configs";
import { Session } from "@/core/ipc/session";
import { ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { LtxInventoryFile } from "@/core/ipc/types/xrf-ltx-inspect";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * Owns the configs project and addresses reads by its committed session identity.
 * Failed replacements leave the current project usable.
 */
@Injectable()
export class ConfigsProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(configsCommands.closeProject);

  /** Whether the backend has been asked what it still has open. */
  @Observable()
  public isReady: boolean = false;

  /** The open project, or null when nothing is open. */
  @Observable()
  public project: AsyncState<ConfigsProjectDescriptor> = AsyncState.idle();

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
    await flowResult(this.restore());
  }

  /**
   * Drop the project on the way out of the application.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();

    runInAction(() => {
      this.project = this.project.asIdle(null);
    });
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
        this.session.open((sessionId) =>
          configsCommands.openProject({
            sessionId,
            isDltx,
            prefix,
            roots: createRoots([root]),
          })
        )
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
    try {
      yield* call(this.session.close());

      this.project = this.project.asIdle(null);
    } catch (error) {
      this.log.error("Failed to close the configs project:", error);
    }
  }

  /**
   * Restores the committed descriptor unless a user action has taken the project flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    try {
      const descriptor: Nullable<ConfigsProjectDescriptor> = yield* call(configsCommands.getProject());

      this.session.adopt(descriptor);
      this.project = this.project.asReady(descriptor);
    } catch (error) {
      this.log.error("Failed to restore the configs project:", error);
    } finally {
      this.isReady = true;
    }
  }
}
