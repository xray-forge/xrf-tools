import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { Nullable } from "@xrf/types";
import { IMenubarItem, IMenubarMenu } from "@xrf/ui/shell/menubar";
import { useCallback } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { KeymapService } from "@/core/keybinds";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { IShellContributed, useShellContributions } from "@/core/shell/contributions";
import { toCommandMenubarItem } from "@/core/shell/menu/components/Menubar";
import { assembleMenus, IAssembledMenu, IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";
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

  const toItem = useCallback(
    (entry: TMenuEntry): IMenubarItem => {
      switch (entry.kind) {
        case "command":
          return toCommandMenubarItem(entry.command, commandsService, keymapService);
        case "action":
          return {
            id: entry.id,
            label: entry.label,
            icon: entry.icon,
            isDisabled: entry.isDisabled,
            onSelect: entry.onSelect,
          };
        case "toggle":
          return {
            id: entry.id,
            label: entry.label,
            isChecked: entry.isChecked,
            isDisabled: entry.isDisabled,
            onSelect: entry.onToggle,
          };
      }
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
    getSections: () => sections.map((entries: ReadonlyArray<TMenuEntry>) => entries.map(toItem)),
  }));
}
