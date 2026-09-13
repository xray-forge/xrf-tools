import { defineCommand, ECommandCategory, ICommandDescriptor } from "@/core/commands";

/** Reads the open sprite again from disk. */
export const RELOAD_EQUIPMENT_SPRITE_COMMAND: ICommandDescriptor = defineCommand({
  id: "sprite-equipment-editor/reload-sprite",
  category: ECommandCategory.APPLICATION,
  chords: ["F5"],
  description: "Reads the open sprite again from disk.",
  label: "Reload sprite",
});

/** Commands reachable inside the equipment sprite editor, beside the root ones. */
export const SPRITE_EQUIPMENT_EDITOR_COMMANDS: ReadonlyArray<ICommandDescriptor> = [RELOAD_EQUIPMENT_SPRITE_COMMAND];
