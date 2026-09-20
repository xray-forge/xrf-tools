import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { TranslationDraft, TTranslationFileEdits } from "@/applications/translations-editor/lib/translation-draft";
import { describeRoots } from "@/core/assets/lib/roots";
import { transformError } from "@/core/error/lib";
import { translationsCommands } from "@/core/ipc/commands/translations";
import { requireSessionId, Session } from "@/core/ipc/session";
import { ETranslationSaveOutcome, SessionSnapshot, TranslationSaveOutcome } from "@/core/ipc/types/xrf-app";
import { TranslationProjectDescriptor, TranslationProjectMode } from "@/core/ipc/types/xrf-translation";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class TranslationsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(translationsCommands.closeProject);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private projectState: AsyncState<SessionSnapshot<TranslationProjectDescriptor>> = AsyncState.idle();

  @Computed()
  public get project(): AsyncState<TranslationProjectDescriptor> {
    return this.projectState.map((snapshot) => snapshot.value);
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
    await flowResult(this.restore());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    const response: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = yield* call(
      translationsCommands.getProject()
    );

    this.log.info(response ? "Existing translations project detected" : "No existing translations project");

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
    this.log.info("Opening translations project:", describeRoots(roots), mode, prefix);

    try {
      this.projectState = this.projectState.asLoading();

      const response: SessionSnapshot<TranslationProjectDescriptor> = yield* call(
        this.session.open((sessionId) => translationsCommands.openProject({ sessionId, roots, mode, prefix }))
      );

      this.log.info("Translations project opened:", Object.keys(response.value.files).length, "files");

      this.projectState = this.projectState.asReady(response);
      this.draft = TranslationDraft.empty();
    } catch (error) {
      this.log.error("Failed to open translations project:", error);

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
   *
   * The refreshed descriptor comes back from the write rather than being patched in here: a save can
   * add or drop entries, and what is on disk is the only version worth showing.
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

      this.projectState = this.projectState.asReady(response.project);

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
