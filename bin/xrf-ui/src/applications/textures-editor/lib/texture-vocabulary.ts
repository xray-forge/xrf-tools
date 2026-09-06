import { TextureVocabularyEntry } from "@/core/bindings/types/xrf-app";

/**
 * The named values to offer for a descriptor field, with an unrecognised stored value appended.
 *
 * A descriptor may hold a number no version of the SDK ever named - a newer converter, or a file edited by hand - and
 * the editor has to be able to show it without pretending it is something else. Appending it keeps the value visible
 * and keeps it saveable; dropping it would silently move the descriptor to the first entry of the list the moment
 * anybody opened the texture, which is the one thing an editor of somebody else's files must never do.
 *
 * @param entries - The named values for the field, from the backend's vocabulary.
 * @param value - The number the descriptor stores.
 * @returns The entries to offer, which is the named ones plus at most one for the value in hand.
 */
export function toVocabularyOptions(
  entries: ReadonlyArray<TextureVocabularyEntry>,
  value: number
): ReadonlyArray<TextureVocabularyEntry> {
  return entries.some((entry: TextureVocabularyEntry) => entry.value === value)
    ? entries
    : [...entries, { label: `Unknown (${value})`, value }];
}
