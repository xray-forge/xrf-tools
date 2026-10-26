import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { configsCommands } from "@xrf/ipc/commands/configs";
import { ConfigsProjectDescriptor } from "@xrf/ipc/types/xrf-app-configs";
import { LtxInventoryFile } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * Owns the configs project; every read of it is addressed by `project.sessionId`. A failed replacement leaves the
 * current project usable.
 */
@Injectable()
export class ConfigsProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly project: SessionRootResource<ConfigsProjectDescriptor> = new SessionRootResource(this.log, {
    subject: "configs project",
    close: configsCommands.closeProject,
    restore: configsCommands.restoreProject,
  });

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
   * Restore whatever project the backend still has open.
   *
   * A reload loses the tree but not the session, so the descriptor comes back with its inventory and the screen
   * returns to where it was rather than to the picker.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.project.restore());
  }

  /**
   * Open a configs tree for browsing.
   *
   * @param root - Filesystem path of a configs directory or a game installation root.
   * @param isDltx - Whether to resolve with the Monolith/Anomaly patch dialect.
   * @param prefix - Logical subtree holding the configs, or null for the whole root.
   */
  @BoundAction()
  public open(root: string, isDltx: boolean, prefix: Nullable<string> = null): Promise<void> {
    return flowResult(
      this.project.open(root, (sessionId) =>
        configsCommands.openProject({ sessionId, isDltx, prefix, roots: createRoots([root]) })
      )
    );
  }

  /**
   * Close the project and go back to the picker.
   */
  @BoundAction()
  public async close(): Promise<void> {
    await flowResult(this.project.close());
  }
}
