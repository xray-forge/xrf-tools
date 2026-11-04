import { Nullable } from "@xrf/types";
import { IMenubarItem } from "@xrf/ui/shell/menubar";

import { IKeybindCommand, KeybindCommandsService } from "@/core/commands";
import { formatChord, KeymapService, parseChord } from "@/core/keybinds";

/**
 * @param command - A command a menu offers.
 * @param commandsService - What runs it and says whether it can run now.
 * @param keymapService - What says the chord that does the same.
 * @returns Its row: its label and icon, its first reachable chord, disabled while it cannot run.
 */
export function toCommandMenubarItem(
  command: IKeybindCommand,
  commandsService: KeybindCommandsService,
  keymapService: KeymapService
): IMenubarItem {
  return {
    id: command.id,
    label: command.label,
    icon: command.icon,
    shortcut: toCommandShortcut(command, keymapService),
    isDisabled: !commandsService.isAvailable(command),
    onSelect: () => commandsService.execute(command),
  };
}

/**
 * @param command - A command a menu row does the same as.
 * @param keymapService - What says the chords reaching it.
 * @returns Its first reachable chord as a row writes it, or null where none reaches it.
 */
export function toCommandShortcut(command: IKeybindCommand, keymapService: KeymapService): Nullable<string> {
  const [chord] = keymapService.getReachableChords(command);

  return chord ? formatChord(parseChord(chord)) : null;
}
