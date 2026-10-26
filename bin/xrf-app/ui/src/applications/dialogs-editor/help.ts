import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const DIALOGS_EDITOR_HELP: IApplicationHelp = {
  summary:
    "Read-only browser for NPC dialog trees. Open a gamedata tree, a game installation, or an authored source tree " +
    "to read every `dialogs*.xml` under `configs\\gameplay` as a graph, with each phrase's line resolved from the " +
    "same root's own string tables. Nothing is written; editing arrives in a later phase.",
  workflow: [
    "Pick a root and a layout. `Game data` reads phrase text from `configs\\text\\<language>`, `Project sources` " +
      "reads it from multi-language JSON under `translations`. The layout is detected and preselected, but never " +
      "decided for you.",
    "Open a dialog from the tree on the left, which groups every dialog under the file declaring it: a double " +
      "click or `Enter` opens one, while a single click only selects its row.",
    "Read the graph. Select a node to see its conditions and effects in the inspector on the right.",
  ],
  nuances: [
    "Reads through the X-Ray virtual filesystem, so an installation opens as readily as a loose tree: on Anomaly " +
      "and CoC the dialogs come out of `db\\configs` and never exist as files on disk.",
    "One dialog per canvas. The project response carries only an index — 502 dialogs' worth of phrases is a " +
      "payload nobody reads — so a dialog is fetched when you open it.",
    "Layout is recomputed on every open rather than saved, so the same dialog always reads the same way. A node " +
      "dragged by hand lasts until the next selection.",
    "A phrase's `<text>` element holds a translation *key*, not the line. The node shows the resolved line where " +
      "there is one and the key where there is not, so an untranslated phrase is visible rather than blank.",
    "A phrase whose line comes from `script_text` has no key to resolve and is not reported as missing one. " +
      "Anomaly does this 417 times.",
    "Numbered edges are the order the player is offered the options, which is game behaviour rather than file " +
      "trivia. A single option carries no number, because there is no order to read.",
    "A quiet rule along a node's bottom edge means the conversation can end there — either because the phrase " +
      "says `is_final` or because it offers nothing, which the engine treats the same way. Roughly four phrases " +
      "in ten end a branch that way, so it marks a shape and not a problem.",
    "A dimmed left edge means nothing in the dialog leads to that phrase, which is why it sits away from the " +
      "conversation rather than in it.",
    "Switching language re-reads the open dialog against an index the backend already holds, so it costs a " +
      "lookup and no file reads.",
  ],
  limitations: [
    "Strictly read-only. No editing, no saving, and no writing back to `dialogs*.xml`.",
    "Script references in `precondition`, `action`, `init_func` and `script_text` are shown but not resolved: " +
      "nothing in the workspace reads Lua `xr_conditions` / `xr_effects` yet, so they are neither confirmed nor " +
      "reported as broken.",
    "Info portions in `has_info` and `give_info` are shown but not checked against the project's `info*.xml`.",
    "No validation pass yet: a phrase referencing an undeclared phrase has its edge dropped from the graph " +
      "rather than reported.",
    "Anomaly and CoC dialog dialects are out of scope, as are the generic dialogs `dialog_manager` builds from " +
      "script at runtime rather than from XML.",
  ],
  relatedTools: [EApplicationId.TRANSLATIONS_EDITOR],
};
