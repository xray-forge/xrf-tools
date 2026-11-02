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
  const [chord] = keymapService.getReachableChords(command);

  return {
    id: command.id,
    label: command.label,
    icon: command.icon,
    shortcut: chord ? formatChord(parseChord(chord)) : null,
    isDisabled: !commandsService.isAvailable(command),
    onSelect: () => commandsService.execute(command),
  };
}
