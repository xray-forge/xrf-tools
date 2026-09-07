import { EventBus, inject, Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { APPLICATION_SOURCE } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/** Writes all pending edits; false keeps the editor open. */
export type EditorSaver = () => Promise<boolean>;

/** Owns navigation blocking and the active document's save/discard/stay sequence. */
@Injectable()
export class EditorLifecycleService {
  @Observable()
  public dirtyCount: number = 0;

  @Observable()
  public isSaving: boolean = false;

  @Observable()
  private busyOwners: ReadonlySet<string> = new Set();

  @Observable()
  private save: Nullable<EditorSaver> = null;

  @Observable()
  private pendingLeave: Nullable<() => void> = null;

  private draftOwner: Nullable<string> = null;
  private savingRequest: Nullable<() => void> = null;

  /** Whether any registered operation or the leave prompt is saving. */
  @Computed()
  public get isBusy(): boolean {
    return this.isSaving || this.busyOwners.size > 0;
  }

  /** Whether a leave request is waiting for a decision. */
  @Computed()
  public get isLeavePending(): boolean {
    return this.pendingLeave !== null;
  }

  /** Whether the active document supplied an operation to save its edits. */
  @Computed()
  public get canSave(): boolean {
    return this.save !== null;
  }

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.busyOwners = new Set();
      this.draftOwner = null;
      this.dirtyCount = 0;
      this.save = null;
      this.pendingLeave = null;
      this.savingRequest = null;
      this.isSaving = false;
    });
  }

  /** Updates one operation's block without clearing another operation's block. */
  @BoundAction()
  public setBusy(owner: string, isBusy: boolean): void {
    if (this.busyOwners.has(owner) === isBusy) {
      return;
    }

    const owners = new Set(this.busyOwners);

    if (isBusy) {
      owners.add(owner);
    } else {
      owners.delete(owner);
    }

    this.busyOwners = owners;
  }

  /** Publishes the active document; replacing its owner invalidates an older leave request. */
  @BoundAction()
  public setDraft(owner: string, dirtyCount: number, save: Nullable<EditorSaver>): void {
    if (this.draftOwner !== owner) {
      this.pendingLeave = null;
    }

    this.draftOwner = owner;
    this.dirtyCount = dirtyCount;
    this.save = save;
  }

  /** Releases only the document belonging to this registration. */
  @BoundAction()
  public releaseDraft(owner: string): void {
    if (this.draftOwner === owner) {
      this.draftOwner = null;
      this.dirtyCount = 0;
      this.save = null;
      this.pendingLeave = null;
    }
  }

  /** Leaves immediately when clean, asks when dirty, and refuses while busy or already asking. */
  @BoundAction()
  public requestLeave(leave: () => void): void {
    if (this.isBusy || this.pendingLeave) {
      return;
    }

    if (this.dirtyCount > 0) {
      // A distinct request identity prevents a late save from completing a newer request.
      this.pendingLeave = () => leave();
    } else {
      leave();
    }
  }

  /** Declines the pending request unless its save is still running. */
  @BoundAction()
  public stay(): void {
    if (!this.isSaving) {
      this.pendingLeave = null;
    }
  }

  /** Executes the pending action once, after the user chose to discard. */
  @BoundAction()
  public discardAndLeave(): void {
    if (this.isBusy) {
      return;
    }

    const leave = this.pendingLeave;

    this.pendingLeave = null;
    leave?.();
  }

  /** Saves once and leaves only on success while the original document and request still exist. */
  @BoundAction()
  public async saveAndLeave(): Promise<void> {
    const request = this.pendingLeave;
    const save = this.save;

    if (this.isBusy || !request || !save) {
      return;
    }

    this.isSaving = true;
    this.savingRequest = request;

    try {
      const isWritten: boolean = await save();

      runInAction(() => {
        if (isWritten && this.pendingLeave === request) {
          this.pendingLeave = null;
          request();
        }
      });
    } catch (error: unknown) {
      if (this.pendingLeave === request) {
        emitNotification(this.eventBus, {
          details: transformError(error).message,
          severity: ENotificationSeverity.ERROR,
          source: APPLICATION_SOURCE,
          title: "Could not save before leaving",
        });
      }
    } finally {
      runInAction(() => {
        if (this.savingRequest === request) {
          this.savingRequest = null;
          this.isSaving = false;
        }
      });
    }
  }
}
