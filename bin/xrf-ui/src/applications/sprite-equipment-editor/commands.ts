import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/** Reads the open sprite again from disk. */
export const RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  id: "sprite-equipment-editor/reload-sprite",
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["F5"],
  description: "Reads the open sprite again from disk.",
  label: "Reload sprite",
});

/** Commands reachable inside the equipment sprite editor, beside the root ones. */
export const SPRITE_EQUIPMENT_EDITOR_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [
  RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND,
];
