import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TRANSLATIONS_PARSER_HELP: IApplicationHelp = {
  summary:
    "Turns raw X-Ray XML string tables into the JSON sources the rest of the translation tooling reads. " +
    "Point it at a downloaded mod, a gamedata tree, or an installed game, name the language its tables " +
    "hold, and every table comes out as one JSON file. Run it once per language and they collect into the " +
    "same files, so one source ends up carrying every language you imported.",
  workflow: [
    "Pick the source. It names a root, not the text directory: the run looks under `configs\\text` when " +
      "the root has one, then descends into the directory named for the language.",
    "Pick the language its tables are written in, and the output directory the JSON sources go to.",
    "`Preview` reports exactly what a run would change without writing anything.",
    "`Import` does the same and writes, reporting what it read, created, updated, and could not read.",
    "Repeat for each further language, into the same output directory.",
  ],
  nuances: [
    "The language is declared, never guessed. Raw XML carries none of its own, and a tree read under the " +
      "wrong key files every string it holds under that language with nothing afterwards to say so. A " +
      "source still holding another language's directory is refused rather than swept up.",
    "Reading goes through the virtual file system, so an installed game works: on Anomaly and CoC the " +
      "tables live inside `db\\configs`, where a plain folder reader finds nothing at all.",
    "Text already in the output is kept when it differs from what was read, and the count of differing " +
      "entries is reported. `Replace existing text that differs` is what takes the imported wording instead.",
    "Ids and language keys come out sorted, so the result does not depend on which language you imported " +
      "first, and importing the same data twice rewrites nothing.",
    "A record missing one of the languages its file carries gets an explicit `null`. Only languages the " +
      "file actually has are filled in this way - importing English and Ukrainian does not declare the " +
      "other six missing.",
    "Text holding the engine's `\\n` line break is stored as an array of lines, which is lossless: the " +
      "build joins it back on the same characters.",
    "A file that cannot be read costs its own strings and nothing else, so one malformed table never stops " +
      "an import. Each one is listed in the findings.",
  ],
  limitations: [
    "One language per run. There is no import-everything button, because nothing in a raw tree reliably " +
      "says which language a given file is.",
    "Merging is additive: an id the XML no longer carries is left in the JSON rather than removed, so a " +
      "source is never pruned to match a mod that dropped a string.",
    "A run that finds no string tables is refused rather than reported as an empty success, so a mistyped " +
      "path fails loudly instead of looking like a clean import that had nothing to do.",
    "No progress and no cancellation. Writes are staged per file, so a failure part-way leaves the files " +
      "already written intact and re-running finishes the job.",
  ],
  relatedTools: [EApplicationId.TRANSLATIONS_EDITOR, EApplicationId.TRANSLATIONS_BUILDER],
};
