import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TRANSLATIONS_VERIFIER_HELP: IApplicationHelp = {
  summary:
    "Reports which translations are missing, and from which languages. It reads authored JSON sources and " +
    "answers one question for each of them: how many ids does this language have " +
    "text for, and how many is it still short. Nothing is written.",
  workflow: [
    "Pick the translations directory, source tree, or installation holding the JSON sources. It starts from the " +
      "configured translations path, or `configs\\text` under game data.",
    "Pick one language, or `all` to report every language the build compiles.",
    "`Verify` answers with a row per file and language, sortable by how much each is missing.",
  ],
  nuances: [
    "An id present with a `null` value counts as missing. That is exactly what a placeholder is - a " +
      "gap somebody left for a translator - so a file full of them reads as incomplete rather than done.",
    "A complete language still gets a row, showing zero missing. A table that listed only failures " +
      "could not tell a finished language apart from one the project does not carry at all.",
    "Reading goes through the virtual file system, so a source tree layered over an installation is " +
      "checked the way the engine would load it rather than the way a directory walk happens to find it.",
    "Only JSON sources are checked. XML under the same tree is a built artifact, not a source.",
    "The check reports counts rather than naming every missing id. Checking a two-language import " +
      "against all eight languages means 149,979 individual gaps - a correct answer, and one no table " +
      "can be read from. The counts say which languages need work and where; `xrf-cli translation " +
      "verify --report` writes the full list when the individual ids are what you need.",
  ],
  limitations: [
    "It does not say which ids are missing, only how many. Use the CLI for the itemised list.",
    "It judges completeness, not correctness: text that is present but wrong, untranslated, or copied " +
      "from another language counts as present.",
    "There is no pass or fail threshold here. `xrf-cli translation verify --strict` is what a build " +
      "step gates on.",
    "No progress and no cancellation. Every source is parsed before anything is reported.",
  ],
  relatedTools: [EApplicationId.TRANSLATIONS_EDITOR, EApplicationId.TRANSLATIONS_PARSER],
};
