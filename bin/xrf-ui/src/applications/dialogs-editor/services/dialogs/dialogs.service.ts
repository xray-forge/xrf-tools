import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describeRoots } from "@/core/assets/lib/roots";
import { dialogsCommands } from "@/core/bindings/commands/dialogs";
import { SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { DialogDescriptor, DialogProjectDescriptor, DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { requireSessionId, Session } from "@/core/ipc/session";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** Which dialog is open, by the pair that addresses one: the file holding it and its id. */
export interface IDialogSelection {
  logicalPath: string;
  id: string;
}

@Injectable()
export class DialogsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(dialogsCommands.closeProject);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private projectState: AsyncState<SessionSnapshot<DialogProjectDescriptor>> = AsyncState.idle();

  @Computed()
  public get project(): AsyncState<DialogProjectDescriptor> {
    return this.projectState.map((snapshot) => snapshot.value);
  }

  /**
   * The dialog being looked at, fetched on selection.
   */
  @Observable()
  public dialog: AsyncState<DialogDescriptor> = AsyncState.idle();

  @Observable()
  public selection: Nullable<IDialogSelection> = null;

  /** Which node of the open dialog the inspector describes. */
  @Observable()
  public inspectedNodeId: Nullable<string> = null;

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
    releaseEditorProject(() => this.session.close(this.projectState.value?.sessionId));
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    const response: Nullable<SessionSnapshot<DialogProjectDescriptor>> = yield* call(dialogsCommands.getProject());

    this.log.info(response ? "Existing dialogs project detected" : "No existing dialogs project");

    this.isReady = true;

    this.projectState = this.projectState.asReady(response);
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
      this.projectState = this.projectState.asLoading();

      const response: SessionSnapshot<DialogProjectDescriptor> = yield* call(
        this.session.open((sessionId) =>
          dialogsCommands.openProject({ sessionId, roots, mode, dialogsPrefix: null, translationsPrefix: null })
        )
      );

      this.log.info(
        "Dialogs project opened:",
        Object.keys(response.value.files).length,
        "files,",
        response.value.textKeys,
        "text keys"
      );

      cancelFlow(this, "dialog");

      this.projectState = this.projectState.asReady(response);
      this.selection = null;
      this.dialog = this.dialog.asIdle();
      this.inspectedNodeId = null;
      this.language = null;
    } catch (error) {
      this.log.error("Failed to open dialogs project:", error);

      this.projectState = this.projectState.asFailed(error as Error);

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
    yield* call(this.session.close(this.projectState.value?.sessionId));

    cancelFlow(this, "dialog");
    this.projectState = this.projectState.asIdle();
    this.dialog = this.dialog.asIdle();
    this.selection = null;
    this.inspectedNodeId = null;
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
    // Only when the dialog actually changes. A language switch re-fetches the same one through here,
    // and dropping the inspection then would close the panel the reader is comparing languages in.
    if (this.selection?.logicalPath !== logicalPath || this.selection.id !== id) {
      this.inspectedNodeId = null;
    }

    this.selection = { id, logicalPath };
    this.dialog = this.dialog.asLoading(null);

    try {
      const response: DialogDescriptor = yield* call(
        dialogsCommands.getDialog({
          sessionId: requireSessionId(this.projectState.value),
          logicalPath,
          id,
          language: this.language,
        })
      );

      this.dialog = this.dialog.asReady(response);
    } catch (error) {
      this.log.error("Failed to read dialog:", logicalPath, id, error);

      this.dialog = this.dialog.asFailed(error as Error, null);
    }
  }

  /** Point the inspector at one node of the open dialog, or at nothing. */
  @BoundAction()
  public inspectNode(nodeId: Nullable<string>): void {
    this.inspectedNodeId = nodeId;
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
