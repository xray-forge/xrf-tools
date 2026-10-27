import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { dialogsCommands } from "@xrf/ipc/commands/dialogs";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { DialogProjectDescriptor, DialogProjectMode } from "@xrf/ipc/types/xrf-dialog";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IDialogTreeEntry, toDialogTreeEntries } from "@/applications/dialogs-editor/lib/dialog-tree";
import { describeRoots } from "@/core/assets/lib/roots";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * The open dialogs project and what it holds; which of its dialogs is shown is `DialogViewService`'s.
 */
@Injectable()
export class DialogsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly project: SessionRootResource<DialogProjectDescriptor> = new SessionRootResource(this.log, {
    subject: "dialogs project",
    close: dialogsCommands.closeProject,
    restore: dialogsCommands.restoreProject,
  });

  /** Whether the backend has been asked what it still holds, which holds the loader back. */
  @Computed()
  public get isReady(): boolean {
    return this.project.isRestored;
  }

  /** Languages the open project's text tree offers, empty when it read none. */
  @Computed()
  public get languages(): Array<string> {
    return this.project.value?.languages ?? [];
  }

  /** Every dialog the open project holds, as a `<file>\<dialog id>` path apiece. */
  @Computed()
  public get entries(): Array<IDialogTreeEntry> {
    return toDialogTreeEntries(this.project.value);
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.project.restore());
  }

  /** Reports the layout roots look like, so the open form can preselect it. */
  public async detectMode(roots: XrayRoots): Promise<Nullable<DialogProjectMode>> {
    try {
      return await dialogsCommands.detectMode(roots);
    } catch (error) {
      this.log.warn("Could not detect dialogs layout:", error);

      return null;
    }
  }

  /**
   * @param roots - Roots the project is read from.
   * @param mode - The layout they hold.
   * @returns Whether the project opened.
   */
  @BoundAction()
  public openProject(roots: XrayRoots, mode: DialogProjectMode): Promise<boolean> {
    return flowResult(
      this.project.open(describeRoots(roots), (sessionId: SessionId) =>
        dialogsCommands.openProject({ sessionId, roots, mode, dialogsPrefix: null, translationsPrefix: null })
      )
    );
  }

  @BoundAction()
  public async closeProject(): Promise<void> {
    await flowResult(this.project.close());
  }
}
