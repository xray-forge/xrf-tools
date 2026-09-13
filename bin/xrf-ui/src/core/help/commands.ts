import { defineCommand, ECommandCategory, ICommandDescriptor } from "@/core/commands";

/** Opens the current tool's authored help. */
export const OPEN_APPLICATION_HELP_COMMAND: ICommandDescriptor = defineCommand({
  category: ECommandCategory.HELP,
  chords: ["F1"],
  description: "Opens help for the tool that is open.",
  id: "help/open",
  label: "Open help",
});

/** Commands help owns, reachable from every application that authored any. */
export const HELP_COMMANDS: ReadonlyArray<ICommandDescriptor> = [OPEN_APPLICATION_HELP_COMMAND];
