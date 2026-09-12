import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TRANSLATIONS_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Compare localization tables across languages and edit target text beside a reference language. " +
    "Open authored project sources or the per-language tables shipped with game data.",
  workflow: [
    "Choose the root containing the translations and check `Layout`: `Project sources` for multi-language JSON " +
      "and language-suffixed XML, or `Game data` for a text directory with language subdirectories.",
    "Open the project, choose a file and the reference and target languages, then filter by id or text if needed.",
    "Edit cells in the target column. Review reported problems and any character encoding errors on edited cells.",
    "Use `Save translations` to write changed files, or `Discard translation edits` to discard all unsaved changes.",
  ],
  nuances: [
    "The root can be project sources, a gamedata tree, or an installation. The suggested layout is detected from " +
      "the path, but you can change it before opening; that choice determines what a save writes.",
    "`not translated` marks an absent value. It is different from a translation whose text is empty.",
    "Edits stay pending until saved. Target text is checked for characters the language's encoding cannot represent.",
    "Saving writes changed files one at a time and stops at the first refused or failed save. Files already saved " +
      "stay written; unsaved edits remain available.",
    "The Problems panel reports issues found while reading the project; a clean read does not mean every id " +
      "has been translated into every language.",
  ],
  limitations: [
    "The table edits target text for existing ids; it does not provide controls to rename ids or create source files.",
    "Saving project sources does not build the per-language game output. Use the translations builder to compile JSON sources.",
  ],
  relatedTools: [
    EApplicationId.TRANSLATIONS_VERIFIER,
    EApplicationId.TRANSLATIONS_BUILDER,
    EApplicationId.TRANSLATIONS_PARSER,
  ],
};
