import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { formatDuration } from "@xrf/format";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/ipc/commands/configs";
import { Session, SessionScopeService } from "@/core/ipc/session";
import { ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app-configs";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import { LtxInventoryFile } from "@/core/ipc/types/xrf-ltx-inspect";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * Owns the configs project and addresses reads by its committed session identity.
 * Failed replacements leave the current project usable.
 */
@Injectable()
export class ConfigsProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(configsCommands.closeProject, inject(SessionScopeService));

  /** Whether the backend has been asked what it still has open. */
  @Observable()
  public isReady: boolean = false;

  @Observable()
  private projectState: AsyncState<SessionSnapshot<ConfigsProjectDescriptor>> = AsyncState.idle();

  /** The open project, or null when nothing is open. */
  @Computed()
  public get project(): AsyncState<ConfigsProjectDescriptor> {
    return this.projectState.map((snapshot) => snapshot.value);
  }

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
    return this.projectState.value?.sessionId ?? null;
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
   * Forget the project on the way out of the application, which the tool's scope closes.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    runInAction(() => {
      this.projectState = this.projectState.asIdle(null);
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
    const timer: Timer = new Timer();

    this.log.info("Opening configs project:", root, { isDltx, prefix });
    this.projectState = this.projectState.asLoading();

    try {
      const snapshot: SessionSnapshot<ConfigsProjectDescriptor> = yield* call(
        this.session.open((sessionId) =>
          configsCommands.openProject({
            sessionId,
            isDltx,
            prefix,
            roots: createRoots([root]),
          })
        )
      );

      this.log.info("Opened configs project:", snapshot.value.root, "in", formatDuration(timer.elapsed()));

      this.projectState = this.projectState.asReady(snapshot);
    } catch (error) {
      this.log.error("Failed to open the configs project:", root, "after", formatDuration(timer.elapsed()), error);

      this.projectState = this.projectState.asFailed(transformError(error));
    } finally {
      this.isReady = true;
    }
  }

  /**
   * Close the project and go back to the picker.
   */
  @LatestFlow("project")
  public *close(): TFlow {
    const root: Nullable<string> = this.project.value?.root ?? null;

    try {
      yield* call(this.session.close());

      this.projectState = this.projectState.asIdle(null);

      if (root) {
        this.log.info("Configs project closed:", root);
      }
    } catch (error) {
      this.log.error("Failed to close the configs project:", root, error);
    }
  }

  /**
   * Restores the committed descriptor unless a user action has taken the project flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    try {
      const snapshot: Nullable<SessionSnapshot<ConfigsProjectDescriptor>> = yield* call(
        configsCommands.restoreProject(this.session.tool)
      );

      this.session.adopt(snapshot);
      this.projectState = this.projectState.asReady(snapshot);

      if (snapshot) {
        this.log.info("Configs project restored:", snapshot.value.root, snapshot.sessionId);
      }
    } catch (error) {
      this.log.error("Failed to restore the configs project:", error);
    } finally {
      this.isReady = true;
    }
  }
}
