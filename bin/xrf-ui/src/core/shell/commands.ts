import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/** Closes the open tool and goes back to all tools, asking first about unsaved work. */
export const CLOSE_TOOL_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["mod+w"],
  description: "Closes the open tool and goes back to all tools, asking first about unsaved work.",
  id: "shell/close-tool",
  label: "Close",
});

/** Commands the shell owns, reachable from every screen. */
export const SHELL_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [CLOSE_TOOL_KEYBIND_COMMAND];
