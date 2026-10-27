import { EToolId, IToolHelp } from "@/core/routing/tool";

export const CONFIGS_FORMATTER_HELP: IToolHelp = {
  summary:
    "Checks or rewrites LTX configuration files into the canonical formatting. Check mode (the default) only " +
    "reports which files differ; format mode rewrites them in place. Semantic validation is not done here - " +
    "that is the verifier's job.",
  workflow: [
    "Pick the configs directory; every `*.ltx` under it, recursively, is included.",
    "Leave `Check only` on to report badly formatted files without writing, or turn it off to rewrite them.",
    "Review the counts and the list of affected files.",
  ],
  nuances: [
    "Canonical form: CRLF line endings, `key = value` spacing, comments preserved but normalized to one space " +
      "after `;`, blank-line runs collapsed with exactly one blank line before each section, inheritance lists " +
      'compacted to `[a]:b,c`, and `#include "path"` spelling.',
    "Nothing is reordered: section order, key order, and duplicates are preserved exactly.",
    "A file that differs only by LF line endings counts as unformatted.",
    "Check mode reads through the virtual filesystem, so configs inside archives are checked too. Format mode " +
      "refuses the whole run if any config exists only in an archive - otherwise it would format the loose " +
      "handful and report success over thousands it never touched.",
    "Files are read as Windows-1251; a file that does not decode fails the run instead of producing a verdict.",
    "`#include` targets are not followed or validated; a broken include does not stop that file from being " +
      "formatted.",
  ],
  limitations: [
    "Does not repair broken syntax - a file that will not parse fails the run rather than being fixed.",
    "No diff preview, no backup, no undo: format mode rewrites in place.",
    "Format mode writes every rewrite at once when the run ends: a file that cannot be read or parsed fails the " +
      "run with nothing written, while a cancelled run writes the files it formatted before it stopped.",
    "Directory-only in this app; formatting individual files is available through `xrf-cli ltx format`.",
  ],
  relatedTools: [EToolId.CONFIGS_VERIFIER],
};
