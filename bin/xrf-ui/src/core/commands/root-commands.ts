import { ICommandDescriptor } from "@/core/commands/lib/command-descriptor";
import { HELP_COMMANDS } from "@/core/help/commands";
import { LAUNCHER_COMMANDS } from "@/core/launcher/commands";

/**
 * Commands reachable from everywhere, whatever is routed.
 */
export const ROOT_COMMANDS: ReadonlyArray<ICommandDescriptor> = [...HELP_COMMANDS, ...LAUNCHER_COMMANDS];
