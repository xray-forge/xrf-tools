import { IKeybindCommand } from "@/core/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";

/** Commands the home screen owns, reachable while nothing is routed. */
export const LAUNCHER_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [FOCUS_SEARCH_KEYBIND_COMMAND];
