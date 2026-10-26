import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const TRANSLATIONS_FORMATTER_HELP: IApplicationHelp = {
  summary:
    "Checks or rewrites JSON translation sources into the canonical formatting. Check mode (the default) only " +
    "reports which files differ; format mode rewrites them in place. Missing translations are not reported here - " +
    "that is the verifier's job.",
  workflow: [
    "Pick the translations directory; every `*.json` under it, recursively, is included.",
    "Leave `Check mode` on to report unformatted sources without writing, or turn it off to rewrite them.",
    "Review the counts and the list of affected files.",
  ],
  nuances: [
    "Canonical form: ids and language keys sorted, two-space indentation, and a trailing newline.",
    "Sorting is natural rather than byte order, matching how rustfmt orders identifiers: `st_thanks2` comes " +
      "before `st_thanks10`, and `ammo-5.45x39` before `ammo-11.43x23`.",
    "Line endings are left alone. Each file keeps the convention it already uses, because that belongs to " +
      "`.gitattributes` rather than to a formatter, and a check does not judge them.",
    "Values are never changed. A one-element array stays an array, a string holding a literal `\\n` stays a " +
      "string, and no `null` placeholder is added for a language a record is missing.",
    "A file already holding the canonical bytes is not rewritten at all, so a run over a clean tree changes no " +
      "timestamps.",
  ],
  limitations: [
    "Does not repair broken JSON - a source that will not parse stops the run rather than being fixed.",
    "No diff preview, no backup, no undo: format mode rewrites in place.",
    "If a source fails mid-run in format mode, files already processed stay rewritten.",
    "Refuses to run while a translations project is open in the editor, because the editor's unsaved views would " +
      "be left stale and the next save would put the pre-format content back. Close the project first.",
    "Directory-only in this app; formatting individual files and asserting line endings are available through " +
      "`xrf-cli translation format`.",
  ],
  relatedTools: [EApplicationId.TRANSLATIONS_VERIFIER, EApplicationId.TRANSLATIONS_EDITOR],
};
