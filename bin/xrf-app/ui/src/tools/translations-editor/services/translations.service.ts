import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { translationsCommands } from "@xrf/ipc/commands/translations";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { ETranslationSaveOutcome, TranslationSaveOutcome } from "@xrf/ipc/types/xrf-app-translations";
import { TranslationProjectDescriptor, TranslationProjectMode } from "@xrf/ipc/types/xrf-translation";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { describeRoots } from "@/core/assets/lib/roots";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { TranslationDraft, TTranslationFileEdits } from "@/tools/translations-editor/lib/translation-draft";

@Injectable()
export class TranslationsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The open project; a save elsewhere that derives it anew is read again. */
  public readonly project: SessionRootResource<TranslationProjectDescriptor> = new SessionRootResource(this.log, {
    subject: "translations project",
    close: translationsCommands.closeProject,
    restore: translationsCommands.restoreProject,
    follow: {
      reread: translationsCommands.restoreProject,
      staleNotice: "Could not read the changed translations project again",
    },
    adopted: () => this.forgetEdits(),
  });

  /**
   * Edits made but not written.
   */
  @Observable()
  private draft: TranslationDraft = TranslationDraft.empty();

  @Observable()
  public savingFile: Nullable<string> = null;

  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  /** Whether a change moved the open project past what is shown, which is being read again. */
  @Computed()
  public get isStale(): boolean {
    return this.project.isStale;
  }

  /**
   * @returns Files with recorded edits, in the order used by Save all.
   */
  @Computed()
  public get dirtyFiles(): Array<string> {
    return this.draft.dirtyFiles;
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.project.restore());
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
    const committed = this.project.value?.files[file]?.entries[id]?.[language] ?? null;

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
    const sessionId: Nullable<SessionId> = this.project.sessionId;

    if (!sessionId) {
      return null;
    }

    try {
      return await translationsCommands.validateText(sessionId, language, text);
    } catch (error) {
      this.log.warn("Could not validate translation text:", error);

      return null;
    }
  }

  /**
   * @param roots - Roots the project is read from.
   * @param mode - The layout they hold.
   * @param prefix - Logical subtree holding the translations, or null for the layout's own.
   * @returns Whether the project opened.
   */
  @BoundAction()
  public openProject(
    roots: XrayRoots,
    mode: TranslationProjectMode,
    prefix: Nullable<string> = null
  ): Promise<boolean> {
    return flowResult(
      this.project.open(describeRoots(roots), (sessionId: SessionId) =>
        translationsCommands.openProject({ sessionId, roots, mode, prefix })
      )
    );
  }

  /**
   * Writes one logical file's pending edits and adopts the project as it is on disk afterwards.
   */
  @LatestFlow("savingFile")
  public *saveFile(file: string): TFlow<boolean> {
    return yield* this.saveFileEdits(file);
  }

  /** Writes pending files sequentially, stopping at the first refused or failed save. */
  @ExclusiveFlow("savingFile")
  public *saveAll(): TFlow<boolean> {
    for (const file of this.dirtyFiles) {
      if (!(yield* this.saveFileEdits(file))) {
        return false;
      }
    }

    return this.dirtyFiles.length === 0;
  }

  @BoundAction()
  public async closeProject(): Promise<void> {
    // A failed close keeps the open project and its edits available for retry.
    if (await flowResult(this.project.close())) {
      this.forgetEdits();
    }
  }

  /** Shares the write between both saves without re-entering their lane. */
  private *saveFileEdits(file: string): TFlow<boolean> {
    const sessionId: Nullable<SessionId> = this.project.sessionId;
    const edits: Nullable<TTranslationFileEdits> = this.draft.toFileEdits(
      file,
      this.project.value?.files[file] ?? null
    );

    // Nothing to write is a save done; edits with no project open to write them into are not.
    if (!edits) {
      return true;
    }

    if (!sessionId) {
      return false;
    }

    this.savingFile = file;

    try {
      const completion: TCommandCompletion<TranslationSaveOutcome> = yield* this.reporter.run({
        name: "Save translations file",
        subject: file,
        command: () => translationsCommands.saveFile(sessionId, file, edits),
        failure: "Could not save translations",
        success: (outcome: TranslationSaveOutcome) =>
          outcome.kind === ETranslationSaveOutcome.STALE
            ? {
                details: `${file}\nThe edits were written, but another project was opened while saving.`,
                severity: ENotificationSeverity.WARNING,
                title: "Saved into a project that is no longer open",
              }
            : null,
      });

      if (completion.error) {
        return false;
      }

      // Only cleared once the write came back: a failed save has to leave the work where it was. A stale save wrote
      // just as much, so the pending work is gone either way - what it does not get to do is say what is open.
      this.discardFile(file);

      if (completion.result.kind === ETranslationSaveOutcome.STALE) {
        return false;
      }

      // A read a later commit's notice started may already show more than this save left, which `accept` keeps.
      this.project.accept(completion.result.project);

      return true;
    } finally {
      // Every path, cancellation included: an open supersedes a save the moment it adopts its project, and the marker
      // would otherwise be left on a file nothing is writing any more.
      this.savingFile = null;
    }
  }

  /** Lets the edits of the project held go, and any save of them still writing, for another project or none. */
  @BoundAction()
  private forgetEdits(): void {
    cancelFlow(this, "savingFile");
    this.draft = TranslationDraft.empty();
  }
}
