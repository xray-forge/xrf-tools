import { ETranslationLanguage } from "@/core/ipc/types/xrf-translation";

/**
 * Languages a translation source can carry, as the backend's `TranslationLanguage` names them, in code order.
 */
export const TRANSLATION_LANGUAGES: ReadonlyArray<ETranslationLanguage> = Object.values(ETranslationLanguage)
  .filter((language: ETranslationLanguage) => language !== ETranslationLanguage.ALL)
  .sort();

/**
 * The language a form starts on when nothing else decides it.
 */
export const DEFAULT_TRANSLATION_LANGUAGE: ETranslationLanguage = ETranslationLanguage.ENGLISH;

/**
 * Stands for every language at once, where a command accepts one.
 */
export const ALL_TRANSLATION_LANGUAGES: ETranslationLanguage = ETranslationLanguage.ALL;

/** Languages offered by commands that also accept an all-language run. */
export const TRANSLATION_LANGUAGES_WITH_ALL: ReadonlyArray<ETranslationLanguage> = [
  ALL_TRANSLATION_LANGUAGES,
  ...TRANSLATION_LANGUAGES,
];
