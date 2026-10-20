import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";
import { LEVEL_KEYBIND_COMMANDS } from "@/core/level/commands";

/** Saves the viewport's finished frame, without its overlays, to the screenshots folder. */
export const TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.VIEW,
  chords: ["F9"],
  description: "Saves the viewport's finished frame, without its overlays, to the screenshots folder.",
  id: "level-viewer/take-screenshot",
  label: "Take screenshot",
});

/** Commands reachable inside the level viewer, beside the root ones. */
export const LEVEL_VIEWER_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [
  ...LEVEL_KEYBIND_COMMANDS,
  TAKE_LEVEL_SCREENSHOT_KEYBIND_COMMAND,
];
