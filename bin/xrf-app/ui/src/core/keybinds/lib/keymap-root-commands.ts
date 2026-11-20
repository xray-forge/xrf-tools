import { InjectionToken } from "@wirestate/core";

import { IKeybindCommand } from "@/core/commands";

/**
 * Commands reachable from every screen, whatever is routed. Bound at the composition root, since the domains declaring
 * them (the shell, the settings dialog, help) draw with the keymap.
 */
export const KEYMAP_ROOT_COMMANDS: InjectionToken<ReadonlyArray<IKeybindCommand>> = new InjectionToken<
  ReadonlyArray<IKeybindCommand>
>("KEYMAP_ROOT_COMMANDS");
