import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { translationsCommands } from "@/core/bindings/commands/translations";
import {
  TranslationEdit,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVariant,
} from "@/core/bindings/types/xrf-translation";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
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

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<TranslationProjectDescriptor>> = createLoadable(null);

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
    releaseEditorProject(translationsCommands.closeProject);
  }

  /**
   * Puts back whatever the backend already had open.
   *
   * Exclusive rather than latest. A restore must lose to anything the user started: joining the lane
   * leaves an open in progress alone, where superseding would cancel the very thing the user asked for. The user's
   * own actions take the lane the other way round, so an open cancels a restore that is still in flight.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    const response: Nullable<TranslationProjectDescriptor> = yield* call(translationsCommands.getProject());

    this.log.info(response ? "Existing translations project detected" : "No existing translations project");

    this.isReady = true;

    if (response) {
      this.project = createLoadable(response);
    }
  }

  /**
   * Reports the layout a directory looks like, so the open form can preselect it.
   */
  public async detectMode(path: string): Promise<Nullable<TranslationProjectMode>> {
    try {
      return await translationsCommands.detectMode(path);
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
    return this.project.value?.files[file]?.entries[id]?.[language] ?? null;
  }

  /** Report the first character a language cannot hold, or `null` when the value is writable. */
  public async validateText(language: string, text: string): Promise<Nullable<string>> {
    try {
      return await translationsCommands.validateText(language, text);
    } catch (error) {
      this.log.warn("Could not validate translation text:", error);

      return null;
    }
  }

  @LatestFlow("project")
  public *openProject(translationsPath: string, mode: TranslationProjectMode): TFlow {
    this.log.info("Opening translations project:", translationsPath, mode);

    try {
      this.project = createLoadable(null, true);

      const response: TranslationProjectDescriptor = yield* call(
        translationsCommands.openProject(translationsPath, mode)
      );

      this.log.info("Translations project opened:", Object.keys(response.files).length, "files");

      this.project = createLoadable(response);
      this.edits = {};
    } catch (error) {
      this.log.error("Failed to open translations project:", error);

      this.project = createLoadable(null, false, error as Error);

      emitNotification(this.eventBus, {
        details: `${translationsPath}
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
  @BoundAction()
  public async saveFile(file: string): Promise<boolean> {
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

    runInAction(() => (this.savingFile = file));

    try {
      const response: TranslationProjectDescriptor = await translationsCommands.saveFile(file, edits);

      runInAction(() => {
        this.project = createLoadable(response);
        this.savingFile = null;
      });

      // Only cleared once the write came back: a failed save has to leave the work where it was.
      this.discardFile(file);

      return true;
    } catch (error) {
      this.log.error("Failed to save translations file:", error);

      runInAction(() => (this.savingFile = null));

      emitNotification(this.eventBus, {
        details: `${file}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_EDITOR,
        title: "Could not save translations",
      });

      return false;
    }
  }

  @LatestFlow("project")
  public *closeProject(): TFlow {
    this.log.info("Closing translations project");

    this.project = this.project.asLoading();

    yield* call(translationsCommands.closeProject());

    this.project = createLoadable(null);

    this.log.info("Translations project closed");
  }
}
