import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describeRoots } from "@/core/assets/lib/roots";
import { dialogsCommands } from "@/core/bindings/commands/dialogs";
import { DialogDescriptor, DialogProjectDescriptor, DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** Which dialog is open, by the pair that addresses one: the file holding it and its id. */
export interface IDialogSelection {
  logicalPath: string;
  id: string;
}

@Injectable()
export class DialogsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<DialogProjectDescriptor>> = createLoadable(null);

  /**
   * The dialog being looked at, fetched on selection.
   */
  @Observable()
  public dialog: Loadable<Nullable<DialogDescriptor>> = createLoadable(null);

  @Observable()
  public selection: Nullable<IDialogSelection> = null;

  /** Which language phrase text is shown in. */
  @Observable()
  public language: Nullable<string> = null;

  /** Languages the open project's text tree offers, empty when it read none. */
  @Computed()
  public get languages(): Array<string> {
    return this.project.value?.languages ?? [];
  }

  /** The language phrase text is actually resolved in, once a dialog has come back. */
  @Computed()
  public get resolvedLanguage(): Nullable<string> {
    return this.dialog.value?.language ?? this.language ?? this.languages[0] ?? null;
  }

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    releaseEditorProject(dialogsCommands.closeProject);
  }

  /**
   * Puts back whatever the backend already had open.
   *
   * Exclusive rather than latest, matching the translations editor: a restore must lose to anything
   * the user started, where superseding would cancel the very open they asked for.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    const response: Nullable<DialogProjectDescriptor> = yield* call(dialogsCommands.getProject());

    this.log.info(response ? "Existing dialogs project detected" : "No existing dialogs project");

    this.isReady = true;

    if (response) {
      this.project = createLoadable(response);
    }
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

  @LatestFlow("project")
  public *openProject(roots: XrayRoots, mode: DialogProjectMode): TFlow {
    this.log.info("Opening dialogs project:", describeRoots(roots), mode);

    try {
      this.project = createLoadable(null, true);

      const response: DialogProjectDescriptor = yield* call(dialogsCommands.openProject(roots, mode, null, null));

      this.log.info(
        "Dialogs project opened:",
        Object.keys(response.files).length,
        "files,",
        response.textKeys,
        "text keys"
      );

      this.project = createLoadable(response);
      this.selection = null;
      this.dialog = createLoadable(null);
      this.language = null;
    } catch (error) {
      this.log.error("Failed to open dialogs project:", error);

      this.project = createLoadable(null, false, error as Error);

      emitNotification(this.eventBus, {
        details: `${describeRoots(roots)}
${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.DIALOGS_EDITOR,
        title: "Could not open dialogs project",
      });
    }
  }

  @LatestFlow("project")
  public *closeProject(): TFlow {
    yield* call(dialogsCommands.closeProject());

    this.project = createLoadable(null);
    this.dialog = createLoadable(null);
    this.selection = null;
    this.language = null;
  }

  /**
   * Fetch one dialog's phrases.
   *
   * Latest rather than exclusive, on its own lane: clicking through a tree starts a fetch per row, and
   * the one that matters is the last one clicked. The selection is set before the call so the tree
   * highlights immediately rather than after the round trip.
   */
  @LatestFlow("dialog")
  public *selectDialog(logicalPath: string, id: string): TFlow {
    this.selection = { id, logicalPath };
    this.dialog = createLoadable(null, true);

    try {
      const response: DialogDescriptor = yield* call(dialogsCommands.getDialog(logicalPath, id, this.language));

      this.dialog = createLoadable(response);
    } catch (error) {
      this.log.error("Failed to read dialog:", logicalPath, id, error);

      this.dialog = createLoadable(null, false, error as Error);
    }
  }

  /**
   * Show phrase text in another language.
   *
   * Re-fetches the open dialog rather than carrying every language in the response: the backend holds
   * the text index in memory, so this costs a lookup and no file reads.
   */
  @BoundAction()
  public setLanguage(language: Nullable<string>): void {
    this.language = language;

    if (this.selection) {
      void flowResult(this.selectDialog(this.selection.logicalPath, this.selection.id));
    }
  }
}
