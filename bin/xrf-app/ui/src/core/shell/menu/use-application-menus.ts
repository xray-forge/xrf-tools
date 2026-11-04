import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { assertExhaustive, Nullable } from "@xrf/types";
import { IMenubarMenu, TMenubarEntry } from "@xrf/ui/shell/menubar";
import { useCallback } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { KeymapService } from "@/core/keybinds";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { IShellContributed, useShellContributions } from "@/core/shell/contributions";
import { toCommandMenubarItem, toCommandShortcut } from "@/core/shell/menu/components/Menubar";
import { assembleMenus, EMenuEntryKind, IAssembledMenu, IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";

/**
 * The window's menus as the menubar draws them: File, the open tool's own menus, then Help, with the sections
 * the tool adds to File and Help, each holding the shell's entries and then whatever the open tool publishes.
 *
 * @returns The menus with something in them, in bar order.
 */
export function useApplicationMenus(): Array<IMenubarMenu> {
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const contributed: IShellContributed<IMenuContribution> = useShellContributions("menus");

  const toSections = useCallback(
    (sections: ReadonlyArray<ReadonlyArray<TMenuEntry>>): Array<Array<TMenubarEntry>> => {
      function toEntry(entry: TMenuEntry): TMenubarEntry {
        switch (entry.kind) {
          case EMenuEntryKind.COMMAND:
            return toCommandMenubarItem(entry.command, commandsService, keymapService);
          case EMenuEntryKind.ACTION:
            return {
              id: entry.id,
              label: entry.label,
              icon: entry.icon,
              isDisabled: entry.isDisabled,
              onSelect: entry.onSelect,
            };
          case EMenuEntryKind.TOGGLE:
            return {
              id: entry.id,
              label: entry.label,
              shortcut: entry.command ? toCommandShortcut(entry.command, keymapService) : null,
              isChecked: entry.isChecked,
              isDisabled: entry.isDisabled,
              onSelect: entry.onToggle,
            };
          case EMenuEntryKind.CHOICE:
            return {
              id: entry.id,
              label: entry.label,
              isChecked: entry.isChosen,
              isExclusive: true,
              isDisabled: entry.isDisabled,
              onSelect: entry.onChoose,
            };
          case EMenuEntryKind.SUBMENU:
            return {
              id: entry.id,
              label: entry.label,
              icon: entry.icon,
              isDisabled: entry.isDisabled,
              sections: toEntrySections(entry.sections),
            };
          default:
            return assertExhaustive(entry);
        }
      }

      function toEntrySections(runs: ReadonlyArray<ReadonlyArray<TMenuEntry>>): Array<Array<TMenubarEntry>> {
        return runs
          .filter((entries: ReadonlyArray<TMenuEntry>) => entries.length > 0)
          .map((entries: ReadonlyArray<TMenuEntry>) => entries.map(toEntry));
      }

      return toEntrySections(sections);
    },
    [commandsService, keymapService]
  );

  return assembleMenus([FILE_MENU, ...(tool?.menus ?? EMPTY_ARRAY), HELP_MENU], tool?.menuExtensions ?? EMPTY_ARRAY, [
    ...contributed.shell,
    ...contributed.tool,
  ]).map(({ menu, sections }: IAssembledMenu) => ({
    id: menu.id,
    label: menu.label,
    // Asked as the menu opens, so a command's guard is read then rather than kept current while nobody looks.
    getSections: () => toSections(sections),
  }));
}
