import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const GAMEDATA_VERIFIER_HELP: IApplicationHelp = {
  summary:
    "Runs every whole-project check over a gamedata tree and reports one verdict per check: configs, meshes, " +
    "textures, sounds, scripts, particles, shaders, spawns, levels and animations. Nothing is written.",
  workflow: [
    "Pick the gamedata directory; it is mounted the way the engine would read it, so archived assets count too.",
    "Start the run and watch the check counter. A long check reports its own progress underneath it.",
    "Read the per-check table: a verdict, how many findings were behind it, and how long it took.",
  ],
  nuances: [
    "A full pass over an installation takes minutes. Opening the project - mounting every root and indexing what " +
      "it declares - is part of that and is inside the elapsed time reported.",
    "`incomplete` is not `failed`. A check that could not read everything it needed has not found a problem; it " +
      "has found that it could not look, and its silence is not a pass.",
    "Stopping lands between checks, never inside one: the checks parallelise internally and have no boundary of " +
      "their own. A stopped run reports the checks that finished and says plainly that the rest never ran.",
    "The run survives leaving this screen and reloading the window; coming back finds it still going.",
    "`Strict` turns findings that would warn into failures, matching `xrf-cli gamedata verify --strict`. " +
      "For textures that is a bump declared without its `bump#` companion, which the game still draws bumped " +
      "without parallax relief, and a bump declaration the game never reads because of the descriptor's type or " +
      "an empty name; a default run counts both and fails only on a bump the game cannot find.",
  ],
  limitations: [
    "Every check runs; there is no way to select a subset here yet. `xrf-cli gamedata verify --checks` can.",
    "Findings are counted rather than listed. A full run produces tens of thousands, which needs a surface built " +
      "to page through them; use the command line with `--report` to read them.",
    "Read-only: nothing here repairs what it reports.",
  ],
  relatedTools: [EApplicationId.CONFIGS_VERIFIER, EApplicationId.ARCHIVES_EXPLORER],
};
