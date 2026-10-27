import { HelpOutlineOutlinedIcon as HelpOutlineIcon } from "@xrf/ui/icons/help-outline-outlined";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

/** Opens the current tool's authored help. */
export const OPEN_APPLICATION_HELP_KEYBIND_COMMAND: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.HELP,
  chords: ["F1"],
  description: "Opens help for the tool that is open.",
  icon: <HelpOutlineIcon />,
  id: "help/open",
  label: "Help",
});

/** Commands help owns, reachable from every application that authored any. */
export const HELP_KEYBIND_COMMANDS: ReadonlyArray<IKeybindCommand> = [OPEN_APPLICATION_HELP_KEYBIND_COMMAND];
