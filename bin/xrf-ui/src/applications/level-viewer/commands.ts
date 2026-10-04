import { IKeybindCommand } from "@/core/commands";
import { LEVEL_KEYBIND_COMMANDS } from "@/core/level/commands";

/** Commands reachable inside the level viewer, beside the root ones. */
export const LEVEL_VIEWER_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = LEVEL_KEYBIND_COMMANDS;
