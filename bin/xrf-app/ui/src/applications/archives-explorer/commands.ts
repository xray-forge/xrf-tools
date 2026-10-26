import { IKeybindCommand } from "@/core/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";

/** Commands reachable inside the archives explorer, beside the root ones. */
export const ARCHIVES_EXPLORER_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [FOCUS_SEARCH_KEYBIND_COMMAND];
