import { Nullable } from "@xrf/types";

import {
  ETranslationEdit,
  TranslationEdit,
  TranslationFile,
  TranslationVariant,
} from "@/core/ipc/types/xrf-translation";

/** The engine's literal line separator, used to display array variants as one editable line. */
const LINE_BREAK: string = "\\n";

type TCellEdits = Readonly<Record<string, Nullable<string>>>;
type TFileEdits = Readonly<Record<string, TCellEdits>>;
type TPendingEdits = Readonly<Record<string, TFileEdits>>;

/** Pending edits for one logical file, grouped by language for the native save command. */
export type TTranslationFileEdits = Record<string, Array<TranslationEdit>>;

/**
 * Immutable pending edits, independent of the committed project and its save lifecycle.
 *
 * A present `null` removes an entry; an absent key leaves its committed value alone.
 */
export class TranslationDraft {
  /**
   * @returns An empty draft, independent of any committed project.
   */
  public static empty(): TranslationDraft {
    return new TranslationDraft({});
  }

  /**
   * @param edits - Immutable records whose file groups each contain at least one edit.
   */
  private constructor(private readonly edits: TPendingEdits) {}

  /**
   * @returns Files with recorded edits, in the enumeration order used by Save all.
   */
  public get dirtyFiles(): Array<string> {
    // Only withEdit creates file groups, and withoutFile removes them whole: no empty groups survive.
    return Object.keys(this.edits);
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @returns Whether the draft overrides the cell's committed value.
   */
  public hasEdit(file: string, language: string, id: string): boolean {
    const pending = this.edits[file]?.[language];

    return Boolean(pending && id in pending);
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @param committed - Current committed value for this cell, or `null` when absent.
   * @returns The pending or committed text, with arrays joined by literal `\n`, or `null` for an absent or removed cell.
   */
  public resolveValue(
    file: string,
    language: string,
    id: string,
    committed: Nullable<TranslationVariant>
  ): Nullable<string> {
    const pending = this.edits[file]?.[language];

    if (pending && id in pending) {
      return pending[id];
    }

    return Array.isArray(committed) ? committed.join(LINE_BREAK) : committed;
  }

  /**
   * @param file - Logical translation file name.
   * @param language - Language containing the cell.
   * @param id - Translation entry identifier.
   * @param value - Replacement text, or `null` to remove the entry; an empty string keeps it present.
   * @returns A new draft with this edit applied, leaving the previous draft unchanged.
   */
  public withEdit(file: string, language: string, id: string, value: Nullable<string>): TranslationDraft {
    return new TranslationDraft({
      ...this.edits,
      [file]: {
        ...this.edits[file],
        [language]: { ...this.edits[file]?.[language], [id]: value },
      },
    });
  }

  /**
   * @param file - Logical translation file name to clear, whether or not it has edits.
   * @returns A new draft retaining the other files' edits, leaving the previous draft unchanged.
   */
  public withoutFile(file: string): TranslationDraft {
    const { [file]: _discarded, ...rest } = this.edits;

    return new TranslationDraft(rest);
  }

  /**
   * @param file - Logical translation file name to save.
   * @param committed - Current committed file, or `null` when absent; missing variants use scalar text.
   * @returns Edits grouped by language, preserving array variants by splitting literal `\n`, or `null` for no edits.
   */
  public toFileEdits(file: string, committed: Nullable<TranslationFile>): Nullable<TTranslationFileEdits> {
    const pending = this.edits[file];

    if (!pending) {
      return null;
    }

    return Object.fromEntries(
      Object.entries(pending).map(([language, byId]) => [
        language,
        Object.entries(byId).map(([id, value]): TranslationEdit => {
          if (value === null) {
            return { kind: ETranslationEdit.REMOVE, id };
          }

          const variant = committed?.entries[id]?.[language];

          return { kind: ETranslationEdit.SET, id, value: Array.isArray(variant) ? value.split(LINE_BREAK) : value };
        }),
      ])
    );
  }
}
