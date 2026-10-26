import { IApplicationHelp } from "@/core/routing/application";

export const EXPORTS_EXPLORER_HELP: IApplicationHelp = {
  summary:
    "Inspect the TypeScript extern declarations in an XRF source tree: their names, signatures, descriptions, " +
    "and source locations. Use it to look up what a script exposes without changing the project.",
  workflow: [
    "Choose the root of the TypeScript script sources and select `Open exports`.",
    "Expand a namespace and double-click a declaration or press `Enter`, or find one with `Filter exports`.",
    "Read its signature and source excerpt. Callable declarations also show their parameter and return details.",
    "Use `Refresh exports` after changing the source files outside the application.",
    "Use `Save exports` to write the declarations out as one of the manifests `xrf-cli externs export` publishes.",
  ],
  nuances: [
    "The tree groups declarations by namespace; the filter can find declarations without expanding those groups.",
    "A project with no extern declarations opens successfully and reports `No externs found`.",
    "A failed refresh keeps the previously loaded declarations visible and reports the failure above them.",
    "The saved file name picks the format: `.json`, `.xml`, or `.html`, and anything else is saved as JSON.",
    "A saved manifest describes the declarations on screen, so refresh first to capture edits made since the open.",
  ],
  limitations: [
    "This is an extern declaration browser, not an inventory of every TypeScript export in a project.",
    "Source excerpts are read-only. The explorer does not edit, build, or execute scripts.",
  ],
};
