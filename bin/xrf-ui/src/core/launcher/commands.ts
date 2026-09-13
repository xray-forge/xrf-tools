import { defineCommand, ECommandCategory, ICommandDescriptor } from "@/core/commands";

/**
 * Puts the caret in the tool search field from anywhere on the home screen.
 *
 * Two chords because they answer different habits, and the bare one is the reason chord-derived suppression exists:
 * `/` inside a field is a character someone is typing, while `mod+k` never is.
 */
export const FOCUS_LAUNCHER_SEARCH_COMMAND: ICommandDescriptor = defineCommand({
  category: ECommandCategory.NAVIGATION,
  chords: ["mod+k", "/"],
  description: "Puts the caret in the tool search field.",
  id: "launcher/focus-search",
  label: "Search tools",
});

/** Commands the home screen owns. */
export const LAUNCHER_COMMANDS: ReadonlyArray<ICommandDescriptor> = [FOCUS_LAUNCHER_SEARCH_COMMAND];
