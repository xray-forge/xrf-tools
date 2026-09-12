import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describeRoots } from "@/core/assets/lib/roots";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { SessionSnapshot, TranslationSaveOutcome } from "@/core/bindings/types/xrf-app";
import {
  TranslationEdit,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVariant,
} from "@/core/bindings/types/xrf-translation";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { requireSessionId, Session } from "@/core/ipc/session";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * How the engine spells a line break inside a string table, and therefore how a multi-line entry is
 * shown as one editable line and split back again.
 */
const LINE_BREAK: string = "\\n";

/** Pending edits for one logical file, grouped by the language each belongs to. */
export type TTranslationFileEdits = Record<string, Array<TranslationEdit>>;

/** `null` marks an entry the user removed, which is not the same as one they blanked. */
export type TPendingValue = Nullable<string>;

/** Uncommitted work, keyed file to language to id. */
export type TPendingEdits = Record<string, Record<string, Record<string, TPendingValue>>>;

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
  public edits: TPendingEdits = {};

  @Observable()
  public savingFile: Nullable<string> = null;

  /** Files holding edits that are not on disk. */
  @Computed()
  public get dirtyFiles(): Array<string> {
    return Object.keys(this.edits).filter((file: string) =>
      Object.values(this.edits[file]).some((byId: Record<string, TPendingValue>) => Object.keys(byId).length > 0)
    );
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
    const response: Nullable<SessionSnapshot<TranslationProjectDescriptor>> = yield* call(
      translationsCommands.getProject()
    );

    this.log.info(response ? "Existing translations project detected" : "No existing translations project");

    this.isReady = true;

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

  /** The value to show for a cell: what is pending if anything is, otherwise what is on disk. */
  public resolveValue(file: string, language: string, id: string): TPendingValue {
    const pending: Optional<Record<string, TPendingValue>> = this.edits[file]?.[language];

    if (pending && id in pending) {
      return pending[id];
    }

    const committed: Nullable<TranslationVariant> = this.committedValue(file, language, id);

    return typeof committed === "string" ? committed : Array.isArray(committed) ? committed.join(LINE_BREAK) : null;
  }

  @BoundAction()
  public setEdit(file: string, language: string, id: string, value: TPendingValue): void {
    this.edits = {
      ...this.edits,
      [file]: {
        ...this.edits[file],
        [language]: { ...this.edits[file]?.[language], [id]: value },
      },
    };
  }

  @BoundAction()
  public discardFile(file: string): void {
    const { [file]: _discarded, ...rest } = this.edits;

    this.edits = rest;
  }

  /**
   * Send an edited value back in the shape the entry already had.
   */
  private toVariant(file: string, language: string, id: string, value: string): TranslationVariant {
    return Array.isArray(this.committedValue(file, language, id)) ? value.split(LINE_BREAK) : value;
  }

  /** What is on disk for a cell, before any pending edit is laid over it. */
  private committedValue(file: string, language: string, id: string): Nullable<TranslationVariant> {
    return this.projectState.value?.value.files[file]?.entries[id]?.[language] ?? null;
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
      this.edits = {};
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
    const pending: Record<string, Record<string, TPendingValue>> | undefined = this.edits[file];

    if (!pending) {
      return true;
    }

    this.log.info("Saving translations file:", file);

    const edits: TTranslationFileEdits = Object.fromEntries(
      Object.entries(pending).map(([language, byId]: [string, Record<string, TPendingValue>]) => [
        language,
        Object.entries(byId).map(([id, value]: [string, TPendingValue]): TranslationEdit => {
          if (value === null) {
            return { kind: "remove", id };
          }

          return { kind: "set", id, value: this.toVariant(file, language, id, value) };
        }),
      ])
    );

    this.savingFile = file;

    try {
      const response: TranslationSaveOutcome = yield* call(
        translationsCommands.saveFile(requireSessionId(this.projectState.value), file, edits)
      );

      // Only cleared once the write came back: a failed save has to leave the work where it was. A stale save wrote
      // just as much, so the pending work is gone either way - what it does not get to do is say what is open.
      this.discardFile(file);

      if (response.kind === "stale") {
        this.log.warn("Translations file saved into a project that is no longer open:", file);

        emitNotification(this.eventBus, {
          details: `${file}\nThe edits were written, but another project was opened while saving.`,
          severity: ENotificationSeverity.WARNING,
          source: EApplicationId.TRANSLATIONS_EDITOR,
          title: "Saved into a project that is no longer open",
        });

        return false;
      }

      this.projectState = this.projectState.asReady(response.project);

      return true;
    } catch (error) {
      this.log.error("Failed to save translations file:", error);

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
      yield* call(this.session.close(this.projectState.value?.sessionId));

      this.projectState = this.projectState.asIdle();
      this.edits = {};

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
