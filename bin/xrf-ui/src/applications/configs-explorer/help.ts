import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const CONFIGS_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Read-only browser for a tree of LTX configs: what the project holds, and what each file says. Open a `configs` " +
    "directory or a whole game installation, whose archived configs are read the way the engine reads them. Nothing " +
    "is ever written.",
  workflow: [
    "Name a configs root - a `configs` directory, or an installation root holding `fsgame.ltx`.",
    "Choose the dialect. `Standard` is vanilla and OpenXRay; `DLTX` applies the `mod_*.ltx` patch files a Monolith " +
      "or Anomaly install carries, which changes what the same files resolve to.",
    "Pick a config in the tree. Its text opens beside it, coloured, with the section headers and includes marked by " +
      "what the parser made of them rather than by what the line looks like.",
  ],
  nuances: [
    "The dialect belongs to the open, not to a toolbar toggle. Everything read in a session was resolved under it, " +
      "so changing it means opening the tree again.",
    "A config is listed by its engine identity, so a file that lives inside a `db` archive appears exactly where the " +
      "engine would find it. Those carry an `archived` badge: they read like any other config and nothing can write " +
      "to them in place.",
    "`entry` marks a config no other config includes, which is the unit that resolves. Almost everything else in a " +
      "game tree is reached through `system.ltx`.",
    "`patch` marks a `mod_*.ltx` file, which only the DLTX dialect recognises. Under standard rules the same file is " +
      "an entry point of its own, and the tree says so.",
    "A section name keeps whatever sits inside its brackets. `[ wpn_base ]` is a different section from `[wpn_base]` " +
      "and nothing inherits it, which is engine behaviour rather than an oversight here.",
  ],
  limitations: [
    "View only. A config editor is separate work; nothing here writes.",
    "A patch file is shown as text alone. Its sections do reach a resolution, folded into the config it patches, but " +
      "nothing records which config that is, so marking them would mean guessing.",
    "A config that will not parse still opens, with its text shown and the parse error marked. What the parser " +
      "could not read is simply absent rather than approximated.",
  ],
  relatedTools: [EApplicationId.CONFIGS_VERIFIER, EApplicationId.CONFIGS_FORMATTER],
};
