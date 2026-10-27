import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { dialogsCommands } from "@xrf/ipc/commands/dialogs";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { DialogDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { CommandOperation } from "@/core/operations/lib/command-operation";
import { IDialogTreeEntry } from "@/tools/dialogs-editor/lib/dialog-tree";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

/** Which dialog is open, by the pair that addresses one: the file holding it and its id. */
export interface IDialogSelection {
  logicalPath: string;
  id: string;
}

/** What the reader looks at, in the project they chose it in. */
interface IDialogView {
  /** The opening of the project it was chosen in; another project shows a view of its own. */
  sessionId: Nullable<SessionId>;
  selection: Nullable<IDialogSelection>;
  /** Which node of the open dialog the inspector describes. */
  inspectedNodeId: Nullable<string>;
  /** Which language phrase text is shown in. */
  language: Nullable<string>;
}

/**
 * Which dialog of the open project is shown and how: the selection, the node inspected, the language, and the dialog
 * read for them. Kept with the project it was chosen in, so a project opened or closed since shows none of it.
 */
@Injectable()
export class DialogViewService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The selected dialog's phrases, fetched on selection. */
  private readonly read: CommandOperation<DialogDescriptor> = new CommandOperation(this.log);

  @Observable()
  private view: IDialogView = { inspectedNodeId: null, language: null, selection: null, sessionId: null };

  /** The opening the read was asked of. */
  @Observable()
  private readFor: Nullable<SessionId> = null;

  public constructor(private readonly dialogsService: DialogsService = inject(DialogsService)) {}

  /** The dialog being looked at, fetched on selection. */
  @Computed()
  public get dialog(): AsyncState<DialogDescriptor> {
    return this.readFor !== null && this.readFor === this.dialogsService.project.sessionId
      ? this.read.state
      : AsyncState.idle();
  }

  @Computed()
  public get selection(): Nullable<IDialogSelection> {
    return this.held.selection;
  }

  /** Which node of the open dialog the inspector describes. */
  @Computed()
  public get inspectedNodeId(): Nullable<string> {
    return this.held.inspectedNodeId;
  }

  /** Which language phrase text is shown in. */
  @Computed()
  public get language(): Nullable<string> {
    return this.held.language;
  }

  /** The language phrase text is actually resolved in, once a dialog has come back. */
  @Computed()
  public get resolvedLanguage(): Nullable<string> {
    return this.dialog.value?.language ?? this.language ?? this.dialogsService.languages[0] ?? null;
  }

  /** @returns The path, or null while nothing is open or the selection names no dialog this project holds. */
  @Computed()
  public get selectedPath(): Nullable<string> {
    const selection: Nullable<IDialogSelection> = this.selection;

    if (!selection) {
      return null;
    }

    return (
      this.dialogsService.entries.find(
        (it: IDialogTreeEntry) => it.payload.id === selection.id && it.payload.logicalPath === selection.logicalPath
      )?.path ?? null
    );
  }

  /** The view of the project held now, empty for one it was not chosen in. */
  @Computed()
  private get held(): IDialogView {
    const sessionId: Nullable<SessionId> = this.dialogsService.project.sessionId;

    return this.view.sessionId === sessionId
      ? this.view
      : { inspectedNodeId: null, language: null, selection: null, sessionId };
  }

  /**
   * Fetch one dialog's phrases.
   *
   * Latest rather than exclusive: clicking through a tree starts a fetch per row, and the one that matters is the last
   * one clicked. The selection is set before the call so the tree highlights immediately rather than after the round
   * trip.
   */
  @LatestFlow("dialog")
  public *selectDialog(logicalPath: string, id: string): TFlow {
    const held: IDialogView = this.held;
    const sessionId: Nullable<SessionId> = held.sessionId;

    if (!sessionId) {
      return;
    }

    // A language switch re-fetches the same dialog through here: it stays drawn and inspected while its text in the
    // other language comes back, or the panel the reader compares languages in would close.
    const isSame: boolean = held.selection?.logicalPath === logicalPath && held.selection.id === id;

    this.view = { ...held, inspectedNodeId: isSame ? held.inspectedNodeId : null, selection: { id, logicalPath } };
    this.readFor = sessionId;

    if (!isSame) {
      this.read.reset();
    }

    yield* this.read.run({
      name: "Read dialog",
      subject: `${logicalPath} ${id}`,
      command: () => dialogsCommands.getDialog({ sessionId, logicalPath, id, language: held.language }),
      failure: "Could not read the dialog",
    });
  }

  /**
   * Close the open dialog without closing the project.
   */
  @BoundAction()
  public clearSelection(): void {
    cancelFlow(this, "dialog");

    this.read.reset();
    this.view = { ...this.held, inspectedNodeId: null, selection: null };
  }

  /** Point the inspector at one node of the open dialog, or at nothing. */
  @BoundAction()
  public inspectNode(nodeId: Nullable<string>): void {
    this.view = { ...this.held, inspectedNodeId: nodeId };
  }

  /**
   * Show phrase text in another language.
   */
  @BoundAction()
  public setLanguage(language: Nullable<string>): void {
    this.view = { ...this.held, language };

    const selection: Nullable<IDialogSelection> = this.selection;

    if (selection) {
      void flowResult(this.selectDialog(selection.logicalPath, selection.id));
    }
  }
}
