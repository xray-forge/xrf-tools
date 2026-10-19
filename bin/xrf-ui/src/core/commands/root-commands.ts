import { IKeybindCommand } from "@/core/commands/lib/command-descriptor";
import { HELP_KEYBIND_COMMANDS } from "@/core/help/commands";
import { SETTINGS_KEYBIND_COMMANDS } from "@/core/settings/commands";
import { SHELL_KEYBIND_COMMANDS } from "@/core/shell/commands";

/**
 * Commands reachable from everywhere, whatever is routed.
 *
 * Deliberately small. A command belongs here only when every screen can answer it; anything a single screen serves
 * is declared by that screen, so it leaves the keymap with it.
 */
export const ROOT_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [
  ...SHELL_KEYBIND_COMMANDS,
  ...SETTINGS_KEYBIND_COMMANDS,
  ...HELP_KEYBIND_COMMANDS,
];
