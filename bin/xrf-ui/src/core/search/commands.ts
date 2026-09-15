import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/**
 * Puts the caret in the search field of whatever surface is open.
 *
 * Declared once and composed into the screens that can serve it, never into the root set: a chord reachable where
 * nothing answers it is a row in Help that lies and a warning in the dev log.
 */
export const FOCUS_SEARCH_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.NAVIGATION,
  chords: ["mod+k", "/"],
  description: "Puts the caret in the search field of the surface that is open.",
  id: "search/focus-field",
  label: "Focus search",
});
