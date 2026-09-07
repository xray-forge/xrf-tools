/**
 * Languages a translation source can carry.
 */
export const TRANSLATION_LANGUAGES: ReadonlyArray<string> = ["eng", "fra", "ger", "ita", "pol", "rus", "spa", "ukr"];

/**
 * The language a form starts on when nothing else decides it.
 */
export const DEFAULT_TRANSLATION_LANGUAGE: string = "eng";

/**
 * Stands for every language at once, where a command accepts one.
 */
export const ALL_TRANSLATION_LANGUAGES: string = "all";

/** Languages offered by commands that also accept an all-language run. */
export const TRANSLATION_LANGUAGES_WITH_ALL: ReadonlyArray<string> = [
  ALL_TRANSLATION_LANGUAGES,
  ...TRANSLATION_LANGUAGES,
];
