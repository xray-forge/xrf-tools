import { InjectionToken } from "@wirestate/core";

import { IKeybindCommand } from "@/core/commands/lib/command-descriptor";
import { IToolHelp } from "@/core/routing/tool";

/**
 * What the home screen declares for itself, as a tool's metadata does: the commands it answers and its help.
 */
export interface ILauncherScreen {
  readonly keybindCommands: ReadonlyArray<IKeybindCommand>;
  readonly help: IToolHelp;
}

/**
 * The home screen's declarations, bound at the composition root so the keymap and help read them without importing
 * the launcher, which draws with both.
 */
export const LAUNCHER_SCREEN: InjectionToken<ILauncherScreen> = new InjectionToken<ILauncherScreen>("LAUNCHER_SCREEN");
