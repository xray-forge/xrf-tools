import { IApplicationHelp } from "@/core/routing/application";

/**
 * Help for the home screen, which is a screen rather than an application and so carries its own.
 */
export const LAUNCHER_HELP: IApplicationHelp = {
  summary: "Find and open XRF tools by group or search. Click `XRF` in the toolbar to return here from a tool.",
  workflow: [
    "Choose a group or type in `Search tools`. Search stays within the selected group; select `All` to search " +
      "every group.",
    "Click a tool to open it. After typing a search, press `Enter` in the search field to open the first match.",
    "Use the view toggle beside the search field to switch between list and grid. Your choice is remembered " +
      "between sessions.",
  ],
  nuances: [
    "Search matches tool names, descriptions and group names, with name matches ranked first. It ignores letter " +
      "case but matches the whole phrase without correcting typos. Try a shorter term if nothing matches.",
    "The counts beside `Tools` describe the selected group or full catalog. The match count above search results " +
      "shows how many tools match your query.",
    "Developer mode also shows tools marked `Planned`. They are not implemented yet; opening one shows its " +
      "intended purpose. With developer mode off, only implemented tools and their groups appear.",
    "Returning home asks for confirmation when a tool has unsaved changes. Navigation is disabled while a " +
      "blocking operation runs.",
  ],
  limitations: [
    "The catalog is fixed for this build: individual tools cannot be reordered, hidden or pinned.",
    "Only one tool is open at a time. Return here to choose another.",
  ],
};
