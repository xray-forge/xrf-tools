import { Window } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-shell";
import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { GitHubIcon } from "@xrf/ui/icons/git-hub";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { IMenubarItem, IMenubarMenu } from "@xrf/ui/shell/menubar";
import { useCallback, useState } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { REPOSITORY_URL } from "@/core/configs";
import { OPEN_TOOL_HELP_KEYBIND_COMMAND } from "@/core/help/commands";
import { KeymapService } from "@/core/keybinds";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { OPEN_SETTINGS_KEYBIND_COMMAND } from "@/core/settings/commands";
import { ESettingsSection } from "@/core/settings/lib/settings-section";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { useEditorMenusRegistry } from "@/core/shell/editor-shell/use-editor-menus";
import { toCommandMenubarItem } from "@/core/shell/menu/components/Menubar";
import { assembleMenus, IAssembledMenu, IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";
import { resolveAppWindow } from "@/lib/tauri/app-window";

/**
 * The window's menus as the menubar draws them: File, the open tool's own menus, then Help, with the sections
 * the tool adds to File and Help, each holding the shell's entries and whatever the open screen publishes.
 *
 * @returns The menus with something in them, in bar order.
 */
export function useApplicationMenus(): Array<IMenubarMenu> {
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const published: ReadonlyArray<IMenuContribution> = useEditorMenusRegistry();
  const [appWindow] = useState<Nullable<Window>>(resolveAppWindow);

  const onOpenRepository = useCallback(() => {
    open(REPOSITORY_URL).catch(Logger.error);
  }, []);

  // Closing asks the window to close, which the page's close guard then answers like the caption's close button.
  const onExit = useCallback(() => {
    appWindow?.close().catch(Logger.error);
  }, [appWindow]);

  const onOpenAbout = useCallback(
    () => settingsDialogService.openSection(ESettingsSection.ABOUT),
    [settingsDialogService]
  );

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

  const shell: Array<IMenuContribution> = [
    {
      section: FILE_MENU.sections.settings,
      entries: [{ kind: "command", command: OPEN_SETTINGS_KEYBIND_COMMAND }],
    },
    {
      section: FILE_MENU.sections.close,
      entries: [
        { kind: "command", command: CLOSE_TOOL_KEYBIND_COMMAND },
        {
          kind: "action",
          id: "shell/exit",
          label: "Exit",
          isDisabled: !appWindow,
          onSelect: onExit,
        },
      ],
    },
    {
      section: HELP_MENU.sections.help,
      entries: [{ kind: "command", command: OPEN_TOOL_HELP_KEYBIND_COMMAND }],
    },
    {
      section: HELP_MENU.sections.about,
      entries: [
        {
          kind: "action",
          id: "shell/open-repository",
          label: "Source",
          icon: <GitHubIcon />,
          onSelect: onOpenRepository,
        },
        {
          kind: "action",
          id: "settings/open-about",
          label: "About",
          icon: <InfoOutlinedIcon />,
          onSelect: onOpenAbout,
        },
      ],
    },
  ];

  return assembleMenus([FILE_MENU, ...(tool?.menus ?? EMPTY_ARRAY), HELP_MENU], tool?.menuExtensions ?? EMPTY_ARRAY, [
    ...shell,
    ...published,
  ]).map(({ menu, sections }: IAssembledMenu) => ({
    id: menu.id,
    label: menu.label,
    // Asked as the menu opens, so a command's guard is read then rather than kept current while nobody looks.
    getSections: () => sections.map((entries: ReadonlyArray<TMenuEntry>) => entries.map(toItem)),
  }));
}
