import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const CONFIGS_VERIFIER_HELP: IApplicationHelp = {
  summary:
    "Checks LTX configs by resolving includes and section inheritance, then validating fields against the " +
    "section schemes declared in the project. Nothing is written.",
  workflow: [
    "Pick the configs directory. LTX files in its subdirectories are included.",
    "Choose `DLTX` for configs that use Monolith/Anomaly patch rules, or `LTX` for standard rules.",
    "Click `Verify`, then review the checked, skipped and invalid section counts. Filter the findings by " +
      "section, field or file to locate a problem.",
  ],
  nuances: [
    "A section scheme defines the rules for a section's fields. Definitions come from `scheme.ltx` and " +
      "`*.scheme.ltx` files; a section selects one with `$scheme`.",
    "Sections without `$scheme` are counted as skipped. A section naming a scheme that cannot be found " +
      "is invalid. Check the skipped count before treating a successful run as full validation.",
    "Included files are read through the configs that include them. The file count reports checked entry " +
      "points, so it can be smaller than the number of LTX files in the directory.",
    "With `DLTX` selected, matching `mod_<base>_*.ltx` files patch their base config instead of being checked " +
      "as separate configs.",
    "Reading uses the virtual filesystem, so configs in archives are included when you select a game tree " +
      "or installation containing them.",
    "You can cancel a run. Stopping takes effect between config entry points, so it may take time; the " +
      "findings returned cover only the part checked before cancellation.",
  ],
  limitations: [
    "Field validation covers only the rules declared by section schemes. A pass does not guarantee that " +
      "the configs will work in the game.",
    "This tool reports problems without fixing or formatting files. Use Configs formatter to check or " +
      "rewrite formatting, or Gamedata verifier for checks across other asset types.",
  ],
  relatedTools: [EApplicationId.CONFIGS_FORMATTER, EApplicationId.GAMEDATA_VERIFIER],
};
