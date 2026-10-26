import { default as GitHubIcon } from "@mui/icons-material/GitHub";
import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { Window } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-shell";
import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { useCallback, useState } from "react";

import { KeybindCommandsService } from "@/core/commands";
import { REPOSITORY_URL } from "@/core/configs";
import { OPEN_APPLICATION_HELP_KEYBIND_COMMAND } from "@/core/help/commands";
import { KeymapService } from "@/core/keybinds";
import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { OPEN_SETTINGS_KEYBIND_COMMAND } from "@/core/settings/commands";
import { ESettingsSection } from "@/core/settings/lib/settings-section";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { CLOSE_TOOL_KEYBIND_COMMAND } from "@/core/shell/commands";
import { useEditorMenusRegistry } from "@/core/shell/editor-shell/use-editor-menus";
import { IMenubarItem, IMenubarMenu, toCommandMenubarItem } from "@/core/shell/menu/components/Menubar";
import { assembleMenus, IAssembledMenu, IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";
import { resolveAppWindow } from "@/lib/tauri/app-window";
import { EMPTY_ARRAY } from "@/lib/types/array";

/**
 * The window's menus as the menubar draws them: File, the open application's own menus, then Help, with the sections
 * the application adds to File and Help, each holding the shell's entries and whatever the open screen publishes.
 *
 * @returns The menus with something in them, in bar order.
 */
export function useApplicationMenus(): Array<IMenubarMenu> {
  const commandsService: KeybindCommandsService = useInjection(KeybindCommandsService);
  const keymapService: KeymapService = useInjection(KeymapService);
  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
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
      entries: [{ kind: "command", command: OPEN_APPLICATION_HELP_KEYBIND_COMMAND }],
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

  return assembleMenus(
    [FILE_MENU, ...(application?.menus ?? EMPTY_ARRAY), HELP_MENU],
    application?.menuExtensions ?? EMPTY_ARRAY,
    [...shell, ...published]
  ).map(({ menu, sections }: IAssembledMenu) => ({
    id: menu.id,
    label: menu.label,
    // Asked as the menu opens, so a command's guard is read then rather than kept current while nobody looks.
    getSections: () => sections.map((entries: ReadonlyArray<TMenuEntry>) => entries.map(toItem)),
  }));
}
