import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/** Drops what is selected in the level, while the scene has the keyboard. */
export const CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["Escape"],
  description: "Drops what is selected in the level, while the scene has the keyboard.",
  id: "level/clear-selection",
  label: "Clear selection",
});

/** Commands the level domain owns, reachable from every application that shows a level. */
export const LEVEL_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND];
