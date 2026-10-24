import { EventBus, inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { formatDuration } from "@xrf/format";
import { assertExhaustive, Nullable } from "@xrf/types";

import { TranslationDraft, TTranslationFileEdits } from "@/applications/translations-editor/lib/translation-draft";
import { describeRoots } from "@/core/assets/lib/roots";
import { transformError } from "@/core/error/lib";
import { translationsCommands } from "@/core/ipc/commands/translations";
import {
  ISessionRevision,
  isOlderSessionAnswer,
  isSessionMovedPast,
  requireSessionId,
  Session,
  SESSION_CHANGE_EVENT,
  SessionScopeService,
} from "@/core/ipc/session";
import {
  ESessionRootChange,
  ETranslationSaveOutcome,
  SessionChange,
  SessionSnapshot,
  TranslationSaveOutcome,
} from "@/core/ipc/types/xrf-app";
import { TranslationProjectDescriptor, TranslationProjectMode } from "@/core/ipc/types/xrf-translation";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

@Injectable()
export class TranslationsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(translationsCommands.closeProject, inject(SessionScopeService));

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private projectState: AsyncState<SessionSnapshot<TranslationProjectDescriptor>> = AsyncState.idle();

  @Computed()
  public get project(): AsyncState<TranslationProjectDescriptor> {
    return this.projectState.map((snapshot) => snapshot.value);
  }

  /** Where the last change derived the project again, which a read of it then catches up with. */
  @Observable()
  private derived: Nullable<ISessionRevision> = null;

  /** Whether a change derived the open project again past what is shown, which is being read again. */
  @Computed()
  public get isStale(): boolean {
    const held: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = this.projectState.value;

    return Boolean(held && this.derived && isSessionMovedPast(held, this.derived));
  }

  /**
   * Edits made but not written.
   */
  @Observable()
  private draft: TranslationDraft = TranslationDraft.empty();

  @Observable()
  public savingFile: Nullable<string> = null;

  /**
   * @returns Files with recorded edits, in the order used by Save all.
   */
  @Computed()
  public get dirtyFiles(): Array<string> {
    return this.draft.dirtyFiles;
  }

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      await flowResult(this.restore());
    } catch (error: unknown) {
      this.log.error("Failed to restore translations project:", error);

      throw error;
    }
  }

  /**
   * Reads the open project again once a change derived it anew, a save elsewhere included; one the backend could not
   * derive again stays as shown, with a notice saying why.
   */
  @OnEvent(SESSION_CHANGE_EVENT)
  public onSessionChange({ payload: change }: WireEvent<SessionChange>): void {
    const held: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = this.projectState.value;

    if (!change || !held || held.sessionId !== change.sessionId) {
      return;
    }

    switch (change.root.kind) {
      // A later notice says what a derivation under way derived.
      case ESessionRootChange.KEPT:
      case ESessionRootChange.DERIVING:
        return;

      case ESessionRootChange.DERIVED:
        if (isSessionMovedPast(held, change)) {
          this.noteDerived(change);
          void this.refresh();
        }

        return;

      case ESessionRootChange.STALE:
      case ESessionRootChange.FAILED:
        this.log.warn("The translations project was not read again:", change.root);

        emitNotification(this.eventBus, {
          details: change.root.kind === ESessionRootChange.FAILED ? change.root.reason : undefined,
          severity: ENotificationSeverity.WARNING,
          source: EApplicationId.TRANSLATIONS_EDITOR,
          title: "Could not read the changed translations project again",
        });

        return;

      default:
        return assertExhaustive(change.root);
    }
  }

  @BoundAction()
  private noteDerived(change: SessionChange): void {
    this.derived = { sessionId: change.sessionId, revision: change.revision };
  }

  /** Reads the committed project again, for `onSessionChange`; a failure leaves what is shown, stale. */
  private async refresh(): Promise<void> {
    try {
      this.adoptRefreshed(await translationsCommands.restoreProject(this.session.tool));
    } catch (error: unknown) {
      this.log.warn("Could not read the translations project again:", transformError(error));
    }
  }

  /**
   * Shows a project read again where it is still the one shown and no older than it; an open or close in flight
   * decides what is shown instead.
   */
  @BoundAction()
  private adoptRefreshed(answer: Nullable<SessionSnapshot<TranslationProjectDescriptor>>): void {
    const held: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = this.projectState.value;

    if (
      answer &&
      held &&
      !this.projectState.isLoading &&
      answer.sessionId === held.sessionId &&
      !isOlderSessionAnswer(held, answer)
    ) {
      this.projectState = this.projectState.map(() => answer);
    }
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    const response: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = yield* call(
      translationsCommands.restoreProject(this.session.tool)
    );

    this.log.info(
      response ? "Translations project restored:" : "No existing translations project",
      response ? describeRoots(response.value.roots) : null
    );

    this.isReady = true;
    this.session.adopt(response);
    this.projectState = this.projectState.asReady(response);
  }

  /**
   * Reports the layout a directory looks like, so the open form can preselect it.
   */
  public async detectMode(roots: XrayRoots): Promise<Nullable<TranslationProjectMode>> {
    try {
      return await translationsCommands.detectMode(roots);
    } catch (error) {
      this.log.warn("Could not detect translations layout:", error);

      return null;
    }
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @returns The text to display, or `null` for an absent or removed cell.
   */
  public resolveValue(file: string, language: string, id: string): Nullable<string> {
    const committed = this.projectState.value?.value.files[file]?.entries[id]?.[language] ?? null;

    return this.draft.resolveValue(file, language, id, committed);
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @returns Whether the current draft overrides the cell's committed value.
   */
  public hasEdit(file: string, language: string, id: string): boolean {
    return this.draft.hasEdit(file, language, id);
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @param value - Replacement text, or `null` to remove the entry; an empty string keeps it present.
   */
  @BoundAction()
  public setEdit(file: string, language: string, id: string, value: Nullable<string>): void {
    this.draft = this.draft.withEdit(file, language, id, value);
  }

  /**
   * @param file - Logical translation file name to clear, whether or not it has edits.
   */
  @BoundAction()
  public discardFile(file: string): void {
    this.draft = this.draft.withoutFile(file);
  }

  /** Report the first character a language cannot hold, or `null` when the value is writable. */
  public async validateText(language: string, text: string): Promise<Nullable<string>> {
    try {
      return await translationsCommands.validateText(requireSessionId(this.projectState.value), language, text);
    } catch (error) {
      this.log.warn("Could not validate translation text:", error);

      return null;
    }
  }

  @LatestFlow("project")
  public *openProject(roots: XrayRoots, mode: TranslationProjectMode, prefix: Nullable<string> = null): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Opening translations project:", describeRoots(roots), mode, prefix);

    try {
      this.projectState = this.projectState.asLoading();

      const response: SessionSnapshot<TranslationProjectDescriptor> = yield* call(
        this.session.open((sessionId) => translationsCommands.openProject({ sessionId, roots, mode, prefix }))
      );

      this.log.info(
        "Translations project opened:",
        describeRoots(roots),
        Object.keys(response.value.files).length,
        "files, in",
        formatDuration(timer.elapsed())
      );

      this.projectState = this.projectState.asReady(response);
      this.draft = TranslationDraft.empty();
    } catch (error) {
      this.log.error(
        "Failed to open translations project:",
        describeRoots(roots),
        mode,
        prefix,
        "after",
        formatDuration(timer.elapsed()),
        error
      );

      this.projectState = this.projectState.asFailed(error as Error);

      emitNotification(this.eventBus, {
        details: `${describeRoots(roots)}
${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_EDITOR,
        title: "Could not open translations project",
      });
    }
  }

  /**
   * Writes one logical file's pending edits and adopts the project as it is on disk afterwards.
   */
  @LatestFlow("project")
  public *saveFile(file: string): TFlow<boolean> {
    return yield* this.saveFileEdits(file);
  }

  /** Writes pending files sequentially, stopping at the first refused or failed save. */
  @ExclusiveFlow("project")
  public *saveAll(): TFlow<boolean> {
    // Every save refreshes the project, so the whole batch holds the project lane.
    for (const file of this.dirtyFiles) {
      if (!(yield* this.saveFileEdits(file))) {
        return false;
      }
    }

    return this.dirtyFiles.length === 0;
  }

  /** Shares the write implementation without re-entering the project's flow lane. */
  private *saveFileEdits(file: string): TFlow<boolean> {
    const edits: Nullable<TTranslationFileEdits> = this.draft.toFileEdits(
      file,
      this.projectState.value?.value.files[file] ?? null
    );

    if (!edits) {
      return true;
    }

    const timer: Timer = new Timer();

    this.log.info("Saving translations file:", file);

    this.savingFile = file;

    try {
      const response: TranslationSaveOutcome = yield* call(
        translationsCommands.saveFile(requireSessionId(this.projectState.value), file, edits)
      );

      // Only cleared once the write came back: a failed save has to leave the work where it was. A stale save wrote
      // just as much, so the pending work is gone either way - what it does not get to do is say what is open.
      this.discardFile(file);

      if (response.kind === ETranslationSaveOutcome.STALE) {
        this.log.warn(
          "Translations file saved into a project that is no longer open:",
          file,
          "in",
          formatDuration(timer.elapsed())
        );

        emitNotification(this.eventBus, {
          details: `${file}\nThe edits were written, but another project was opened while saving.`,
          severity: ENotificationSeverity.WARNING,
          source: EApplicationId.TRANSLATIONS_EDITOR,
          title: "Saved into a project that is no longer open",
        });

        return false;
      }

      const held: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = this.projectState.value;

      // A read a later commit's notice started may already show more than this save left.
      this.projectState = this.projectState.asReady(
        held && isOlderSessionAnswer(held, response.project) ? held : response.project
      );

      this.log.info("Translations file saved:", file, "in", formatDuration(timer.elapsed()));

      return true;
    } catch (error) {
      this.log.error("Failed to save translations file:", file, "after", formatDuration(timer.elapsed()), error);

      emitNotification(this.eventBus, {
        details: `${file}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_EDITOR,
        title: "Could not save translations",
      });

      return false;
    } finally {
      // Every path, cancellation included: this lane cancels a save the moment an open supersedes it, and the marker
      // would otherwise be left on a file nothing is writing any more.
      this.savingFile = null;
    }
  }

  @LatestFlow("project")
  public *closeProject(): TFlow {
    this.log.info("Closing translations project");

    const previous = this.projectState;
    const loading = previous.asLoading();

    this.projectState = loading;

    try {
      yield* call(this.session.close());

      this.projectState = this.projectState.asIdle();
      this.draft = TranslationDraft.empty();

      this.log.info("Translations project closed");
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close translations project:", transformed);

      emitNotification(this.eventBus, {
        details: transformed.message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_EDITOR,
        title: "Could not close translations project",
      });
    } finally {
      // A failed or abandoned close keeps the open project and its edits available for retry.
      if (this.projectState === loading) {
        this.projectState = previous;
      }
    }
  }
}
