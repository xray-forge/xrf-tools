import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TRANSLATIONS_BUILDER_HELP: IApplicationHelp = {
  summary:
    "Compiles JSON translation sources into the string tables the game loads: one XML " +
    "file per source per language, written in that language's code page. This is the last step " +
    "before packaging, and the only one that produces files the engine reads directly.",
  workflow: [
    "Pick the translations directory, source tree, or installation holding the JSON sources. It starts from the " +
      "configured translations path, or `configs\\text` under game data.",
    "Pick a language, or `all` to compile every language the build supports.",
    "Pick the output directory. Tables land under `<output>/<language>/<name>.xml`.",
    "`Build` reports a row per language with how many tables it wrote and how many ids each holds.",
  ],
  nuances: [
    "A missing translation compiles to the id itself, which is the engine's own fallback, so every " +
      "language gets a complete table rather than a short one. An untranslated string shows its key " +
      "in game instead of showing nothing - use the verifier to find them before shipping.",
    "Each language is encoded in its own code page: windows-1251 for Russian and Ukrainian, " +
      "windows-1250 for German and Polish, windows-1252 for the rest. A character the target cannot " +
      "represent fails the build rather than being written as a replacement.",
    "Ids are sorted by default. Turning that off preserves the order each source declares them in, " +
      "which matters only when you are diffing built output against something produced elsewhere.",
    "Sources are read through the virtual file system, so a tree layered over an installation " +
      "compiles what the engine would actually load. Output is always a plain directory, because a " +
      "string table is a file and an archive has nowhere to put one.",
    "Two sources that would write the same table fail the build before anything is written, rather " +
      "than letting one quietly overwrite the other.",
  ],
  limitations: [
    "The output directory may not sit inside any of the source roots, so a build cannot fill an " +
      "authored tree with generated files.",
    "Existing tables at the destination are overwritten without asking, and nothing already there is " +
      "removed first - an older build of sources you have since renamed stays mixed in with this one.",
    "Only JSON sources are compiled. XML under the same tree is built output, not a source.",
    "No progress and no cancellation. The first failure ends the run, and whatever reached disk " +
      "before it stays there.",
  ],
  relatedTools: [EApplicationId.TRANSLATIONS_VERIFIER, EApplicationId.TRANSLATIONS_EDITOR],
};
